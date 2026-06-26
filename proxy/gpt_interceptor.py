"""
gpt_interceptor.py — mitmproxy addon
Intercepta responses de chatgpt.com y extrae headers de rate limit.
Escribe el estado en ~/.gptrouter/usage.json para que la app Swift lo lea.
"""

import json
import hashlib
import os
import time
import re
from datetime import datetime
from mitmproxy import http

STORAGE_PATH = os.path.expanduser("~/.gptrouter/usage.json")
ACCOUNTS_PATH = os.path.expanduser("~/.gptrouter/accounts.json")

# Headers conocidos que ChatGPT devuelve con info de rate limit
RATE_LIMIT_HEADERS = [
    "x-ratelimit-limit-requests",
    "x-ratelimit-limit-tokens",
    "x-ratelimit-remaining-requests",
    "x-ratelimit-remaining-tokens",
    "x-ratelimit-reset-requests",
    "x-ratelimit-reset-tokens",
    # Headers alternativos que OpenAI usa en algunos endpoints
    "openai-processing-ms",
    "x-request-id",
]

# Endpoints de ChatGPT que tienen info útil
RELEVANT_PATHS = [
    "/backend-api/conversation",
    "/backend-api/conversations",
    "/backend-api/me",
    "/backend-api/usage",
    "/backend-api/wham/usage",
    "/backend-api/models",
    "/v1/chat/completions",
]


def load_state() -> dict:
    """Carga el estado actual del archivo JSON."""
    try:
        with open(STORAGE_PATH, "r") as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        return {"accounts": {}, "last_updated": None}


def account_has_usage(account: dict) -> bool:
    return bool(account.get("weekly")) or bool(account.get("hourly"))


def account_dedupe_key(account: dict) -> str:
    email = account.get("email")
    if email:
        return f"email:{email.lower()}"
    account_id = account.get("account_id") or account.get("id")
    return f"id:{account_id}"


def pick_better_account(current, candidate: dict) -> dict:
    if current is None:
        return candidate

    current_has_usage = account_has_usage(current)
    candidate_has_usage = account_has_usage(candidate)
    if candidate_has_usage != current_has_usage:
        return candidate if candidate_has_usage else current

    return candidate if str(candidate.get("last_request") or "") >= str(current.get("last_request") or "") else current


def dedupe_state(state: dict) -> dict:
    deduped = {}
    for account in state.get("accounts", {}).values():
        key = account_dedupe_key(account)
        deduped[key] = pick_better_account(deduped.get(key), account)

    state["accounts"] = {
        account.get("id") or key: account
        for key, account in deduped.items()
    }
    return state


def save_state(state: dict):
    """Guarda el estado en el archivo JSON."""
    os.makedirs(os.path.dirname(STORAGE_PATH), exist_ok=True)
    state = dedupe_state(state)
    state["last_updated"] = datetime.now().isoformat()
    with open(STORAGE_PATH, "w") as f:
        json.dump(state, f, indent=2)


def parse_reset_time(reset_str: str) -> str:
    """
    Convierte strings de reset como '5h30m12s' o '2026-01-15T10:00:00Z'
    a un ISO timestamp legible.
    """
    if not reset_str:
        return None

    # Si ya es un timestamp ISO
    if "T" in reset_str or "-" in reset_str:
        return reset_str

    # Formato: '5h30m12s', '2m30s', '45s'
    total_seconds = 0
    pattern = r'(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?'
    match = re.match(pattern, reset_str)
    if match:
        h, m, s = match.groups()
        total_seconds = int(h or 0) * 3600 + int(m or 0) * 60 + int(s or 0)
        reset_at = time.time() + total_seconds
        return datetime.fromtimestamp(reset_at).isoformat()

    return reset_str


def extract_account_from_cookie(cookies: str) -> str:
    """
    Intenta identificar la cuenta desde las cookies de la request.
    Retorna un identificador de sesión simplificado.
    """
    if not cookies:
        return "default"
    # Busca __Secure-next-auth.session-token o similar
    match = re.search(r'__Host-next-auth\.csrf-token=([^;]+)', cookies)
    if match:
        # Usa los primeros 8 chars como ID de cuenta
        return match.group(1)[:8]
    # Fallback estable entre reinicios del proxy
    return hashlib.sha256(cookies[:100].encode("utf-8")).hexdigest()[:8]


def _walk_usage_values(value, context=""):
    """Yield (context, key, value) for nested usage JSON without storing secrets."""
    if isinstance(value, dict):
        for key, child in value.items():
            next_context = f"{context}.{key}".lower() if context else str(key).lower()
            yield from _walk_usage_values(child, next_context)
    elif isinstance(value, list):
        for child in value:
            yield from _walk_usage_values(child, context)
    else:
        yield context, context.rsplit(".", 1)[-1], value


def apply_usage_body(account: dict, body: dict):
    """Best-effort parser for current ChatGPT/Codex usage payloads."""
    account.pop("usage_raw", None)
    account["usage_raw_keys"] = list(body.keys())

    if body.get("email"):
        account["email"] = body["email"]

    rate_limit = body.get("rate_limit")
    if isinstance(rate_limit, dict):
        primary = rate_limit.get("primary_window")
        secondary = rate_limit.get("secondary_window")

        if isinstance(primary, dict) and primary.get("used_percent") is not None:
            used = max(0, min(100, int(primary["used_percent"])))
            account["hourly"]["limit_tokens"] = 100
            account["hourly"]["remaining_tokens"] = 100 - used
            if primary.get("reset_at"):
                account["hourly"]["resets_at"] = datetime.fromtimestamp(int(primary["reset_at"])).isoformat()
            elif primary.get("reset_after_seconds"):
                reset_at = time.time() + int(primary["reset_after_seconds"])
                account["hourly"]["resets_at"] = datetime.fromtimestamp(reset_at).isoformat()

        if isinstance(secondary, dict) and secondary.get("used_percent") is not None:
            used = max(0, min(100, int(secondary["used_percent"])))
            account["weekly"]["limit_requests"] = 100
            account["weekly"]["remaining_requests"] = 100 - used
            if secondary.get("reset_at"):
                account["weekly"]["resets_at"] = datetime.fromtimestamp(int(secondary["reset_at"])).isoformat()
            elif secondary.get("reset_after_seconds"):
                reset_at = time.time() + int(secondary["reset_after_seconds"])
                account["weekly"]["resets_at"] = datetime.fromtimestamp(reset_at).isoformat()

    for context, key, value in _walk_usage_values(body):
        if not isinstance(value, (int, float, str)):
            continue

        key_context = f"{context}.{key}".lower()
        is_weekly = any(token in key_context for token in ["week", "weekly"])
        is_hourly = any(token in key_context for token in ["5h", "five", "hour", "hourly"])

        if not (is_weekly or is_hourly):
            continue

        target = account["weekly"] if is_weekly else account["hourly"]
        number_value = None
        if isinstance(value, (int, float)):
            number_value = int(value)
        elif isinstance(value, str) and value.isdigit():
            number_value = int(value)

        if number_value is not None:
            if any(token in key_context for token in ["remaining", "remain", "available", "left"]):
                target["remaining_requests" if is_weekly else "remaining_tokens"] = number_value
            elif any(token in key_context for token in ["limit", "total", "max", "quota"]):
                target["limit_requests" if is_weekly else "limit_tokens"] = number_value
            elif any(token in key_context for token in ["used", "usage", "consumed", "count"]):
                target["messages_sent" if is_weekly else "messages_sent_5h"] = number_value
        elif isinstance(value, str) and any(token in key_context for token in ["reset", "resets", "until"]):
            target["resets_at"] = parse_reset_time(value)


class GPTInterceptor:
    """Addon de mitmproxy para capturar uso de ChatGPT."""

    def __init__(self):
        self.state = load_state()
        print(f"[GPT Router] Proxy iniciado. Estado en: {STORAGE_PATH}")

    def response(self, flow: http.HTTPFlow) -> None:
        """Llamado en cada response HTTP que pasa por el proxy."""
        host = flow.request.pretty_host

        # Solo nos interesan requests a ChatGPT/OpenAI
        if not any(domain in host for domain in ["chatgpt.com", "api.openai.com", "openai.com"]):
            return

        path = flow.request.path
        
        # Filtrar endpoints relevantes
        if not any(path.startswith(p) for p in RELEVANT_PATHS):
            return

        # Extraer identificador de cuenta desde cookies
        cookies = flow.request.headers.get("cookie", "")
        account_id = extract_account_from_cookie(cookies)

        # Inicializar cuenta si no existe
        if account_id not in self.state["accounts"]:
            self.state["accounts"][account_id] = {
                "id": account_id,
                "first_seen": datetime.now().isoformat(),
                "weekly": {},
                "hourly": {},
                "last_request": None,
                "endpoint_hits": {}
            }

        account = self.state["accounts"][account_id]
        account["last_request"] = datetime.now().isoformat()

        # Contar hits por endpoint
        endpoint_key = path.split("?")[0]
        account["endpoint_hits"][endpoint_key] = account["endpoint_hits"].get(endpoint_key, 0) + 1

        # --- Extraer rate limit headers ---
        headers = flow.response.headers if flow.response else {}
        rl_data = {}

        for header in RATE_LIMIT_HEADERS:
            value = headers.get(header) or headers.get(header.lower())
            if value:
                rl_data[header] = value

        if rl_data:
            print(f"[GPT Router] Rate limit headers encontrados en {path}:")
            for k, v in rl_data.items():
                print(f"  {k}: {v}")

            # Mapear a estructura normalizada
            if "x-ratelimit-remaining-requests" in rl_data:
                account["weekly"]["remaining_requests"] = int(rl_data["x-ratelimit-remaining-requests"])

            if "x-ratelimit-limit-requests" in rl_data:
                account["weekly"]["limit_requests"] = int(rl_data["x-ratelimit-limit-requests"])

            if "x-ratelimit-reset-requests" in rl_data:
                account["weekly"]["resets_at"] = parse_reset_time(rl_data["x-ratelimit-reset-requests"])

            if "x-ratelimit-remaining-tokens" in rl_data:
                account["hourly"]["remaining_tokens"] = int(rl_data["x-ratelimit-remaining-tokens"])

            if "x-ratelimit-limit-tokens" in rl_data:
                account["hourly"]["limit_tokens"] = int(rl_data["x-ratelimit-limit-tokens"])

            if "x-ratelimit-reset-tokens" in rl_data:
                account["hourly"]["resets_at"] = parse_reset_time(rl_data["x-ratelimit-reset-tokens"])

        # --- Parsear body de /backend-api/me para obtener email ---
        if "/backend-api/me" in path and flow.response and flow.response.status_code == 200:
            try:
                body = json.loads(flow.response.get_text())
                account["email"] = body.get("email", "")
                account["name"] = body.get("name", "")
                account["plan"] = body.get("plan", {})
                print(f"[GPT Router] Usuario identificado: {account.get('email')}")
            except Exception:
                pass

        # --- Parsear endpoints de uso actuales ---
        if ("/backend-api/usage" in path or "/backend-api/wham/usage" in path) and flow.response and flow.response.status_code == 200:
            try:
                body = json.loads(flow.response.get_text())
                apply_usage_body(account, body)
                print(f"[GPT Router] Datos de uso capturados: {list(body.keys())}")
            except Exception:
                pass

        # --- Contar mensajes enviados manualmente ---
        if "/backend-api/conversation" in path and flow.request.method == "POST":
            account["weekly"]["messages_sent"] = account["weekly"].get("messages_sent", 0) + 1
            account["hourly"]["messages_sent_5h"] = account["hourly"].get("messages_sent_5h", 0) + 1

            # Marcar timestamp del último mensaje
            account["weekly"]["last_message_at"] = datetime.now().isoformat()

            # Parsear modelo usado
            try:
                req_body = json.loads(flow.request.get_text())
                model = req_body.get("model", "unknown")
                if "models" not in account:
                    account["models"] = {}
                account["models"][model] = account["models"].get(model, 0) + 1
            except Exception:
                pass

        self.state["accounts"][account_id] = account
        save_state(self.state)


addons = [GPTInterceptor()]
