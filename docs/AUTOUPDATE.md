# Autoupdate de Kuota

Kuota usa dos fuentes de actualizacion:

1. **Kuota app** mediante Tauri Updater y GitHub Releases.
2. **codex-auth** mediante `npm install -g @loongphy/codex-auth@latest`.

## Estado actual

Ya esta integrado:

- `@tauri-apps/plugin-updater`
- `@tauri-apps/plugin-process`
- `tauri-plugin-updater`
- `tauri-plugin-process`
- permisos `updater:default` y `process:allow-restart`
- generacion de updater artifacts con `bundle.createUpdaterArtifacts = true`
- workflow `.github/workflows/release.yml` para compilar releases por tag
- UI preparada para buscar actualizaciones desde `Ajustes`
- endpoint real de GitHub Releases:
  `https://github.com/richtunic/Kuota/releases/latest/download/latest.json`
- public key configurada en `src-tauri/tauri.conf.json`
- secrets creados en GitHub:
  - `TAURI_SIGNING_PRIVATE_KEY`
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

## Seguridad de firma

Tauri Updater requiere firma obligatoria. La llave privada ya esta guardada como secret de GitHub y no debe commitearse.

Si se pierde la private key o su password, las actualizaciones firmadas con esa llave dejaran de poder publicarse. En ese caso habria que rotar la llave y publicar una nueva version con la public key actualizada.

## Rotar llave de firma

Solo si se necesita rotar la llave:

```bash
npx tauri signer generate -w ~/.tauri/kuota.key
```

Guardar la nueva llave privada en GitHub Secrets:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

Y reemplazar la public key en `src-tauri/tauri.conf.json`.

## Endpoint de GitHub Releases

El updater apunta a:

```txt
https://github.com/richtunic/Kuota/releases/latest/download/latest.json
```

La configuracion vive en `src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "pubkey": "...",
      "endpoints": [
        "https://github.com/richtunic/Kuota/releases/latest/download/latest.json"
      ]
    }
  }
}
```

## Crear release

1. Actualizar version en `package.json`, `src-tauri/Cargo.toml` y `src-tauri/tauri.conf.json`.
2. Crear tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

3. GitHub Actions crea una release draft con instaladores y artifacts firmados.
4. Revisar la release.
5. Publicarla.

## Flujo dentro de la app

Al presionar `Buscar actualizaciones` en Kuota:

1. Kuota actualiza/verifica `codex-auth`.
2. Kuota consulta el updater de Tauri.
3. Si hay una version nueva de Kuota:
   - descarga el paquete firmado;
   - lo instala;
   - relanza la app.

Si el endpoint o la llave publica no estuvieran configurados, Kuota muestra un mensaje controlado indicando que el actualizador esta pendiente de configurar.
