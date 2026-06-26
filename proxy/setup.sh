#!/bin/bash
# setup.sh — Instala mitmproxy y configura el certificado para HTTPS
# Ejecutar UNA SOLA VEZ antes de usar GPT Router

set -e

echo "🔧 GPT Router — Setup inicial"
echo "================================"

# 1. Instalar mitmproxy si no existe
if ! command -v mitmdump &> /dev/null; then
    echo "📦 Instalando mitmproxy..."
    pip3 install mitmproxy --break-system-packages 2>/dev/null || pip3 install mitmproxy
    echo "✅ mitmproxy instalado"
else
    echo "✅ mitmproxy ya está instalado: $(mitmdump --version | head -1)"
fi

# 2. Crear directorio de estado
mkdir -p ~/.gptrouter
echo '{"accounts": {}, "last_updated": null}' > ~/.gptrouter/usage.json
echo "✅ Directorio ~/.gptrouter creado"

# 3. Generar certificado corriendo mitmproxy brevemente
echo "🔐 Generando certificado SSL de mitmproxy..."
timeout 3 mitmdump --quiet 2>/dev/null || true

CERT_PATH="$HOME/.mitmproxy/mitmproxy-ca-cert.pem"
if [ -f "$CERT_PATH" ]; then
    echo "✅ Certificado generado en: $CERT_PATH"
    
    # 4. Instalar certificado en el Keychain de macOS
    echo "🔐 Instalando certificado en macOS Keychain..."
    echo "   (Se pedirá tu contraseña del sistema)"
    
    sudo security add-trusted-cert \
        -d \
        -r trustRoot \
        -k /Library/Keychains/System.keychain \
        "$CERT_PATH"
    
    echo "✅ Certificado instalado en el Keychain del sistema"
else
    echo "⚠️  No se encontró el certificado. Ejecuta mitmproxy manualmente una vez."
fi

echo ""
echo "================================"
echo "✅ Setup completado"
echo ""
echo "IMPORTANTE: También necesitas instalar el certificado en Chrome:"
echo "  1. Chrome → Settings → Privacy → Security → Manage certificates"
echo "  2. Importa: $HOME/.mitmproxy/mitmproxy-ca-cert.pem"
echo "  O ejecuta:"
echo "  open $HOME/.mitmproxy/mitmproxy-ca-cert.pem"
echo ""
echo "Ahora puedes abrir GPT Router.app"
