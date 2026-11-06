<p align="center">
  <img src="docs/images/poki-icon.png" width="72" height="72" alt="Icono de Poki">
</p>

<h1 align="center">Poki for Linux</h1>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-CN.md">简体中文</a> | <a href="README.zh-TW.md">繁體中文</a> | <a href="README.ja.md">日本語</a> | <a href="README.ko.md">한국어</a> | Español
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0--only-blue" alt="Licencia: AGPL-3.0-only"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20iOS-lightgrey" alt="Plataforma: Linux e iOS">
</p>

**Controla un iPhone real desde un agente MCP en un equipo Linux: sin Mac, sin jailbreak y sin hardware adicional.** Poki expone el árbol de accesibilidad del iPhone y su entrada HID (tocar, deslizar, escribir, teclas) a cualquier cliente MCP a través de un servidor local, para que un agente opere el teléfono igual que lo harías tú.

> Este repositorio es una **adaptación a Linux no oficial** de [JosssphZhou/poki](https://github.com/JosssphZhou/poki). El soporte de anfitrión macOS en el que se basa el proyecto original —Xcode, el asistente OCR de Vision, el simulador— aquí se reemplaza o se omite; todo lo que toca el dispositivo es el diseño original, impulsado por herramientas de Linux. Las notas detalladas de la adaptación están en [LINUX.md](LINUX.md). Sitio del proyecto original: https://poki.bot

## Cómo funciona

Un puente escrito en Rust y basado en [idevice](https://github.com/jkcoxson/idevice) se comunica con el iPhone por USB a través de **usbmuxd**, abre el túnel de CoreDevice y habla RemoteXPC con los servicios de desarrollador: el mismo camino que usa Xcode, sin Xcode. La imagen de disco de desarrollador la instala **pymobiledevice3**, que la descarga y la personaliza para el dispositivo. El servidor MCP en sí es un programa Node.js puro.

## Requisitos

- Un equipo Linux con usbmuxd en funcionamiento (verificado en Arch Linux, x86_64)
- Un iPhone con el Modo de desarrollador y la Automatización de la interfaz activados (consulta [Configuración de un iPhone físico](docs/physical-iphone.md); los pasos en el dispositivo son los mismos que en el proyecto original)
- Node.js 24+, Rust (vía rustup), Python 3, ffmpeg, git
- pymobiledevice3 (`pip install pymobiledevice3`, o en un entorno virtual; apunta `POKI_PYMOBILEDEVICE3` a su ubicación si no está en el `PATH`)

`./bin/setup` comprueba los requisitos que falten y dice cómo instalar cada uno.

## Inicio rápido

```sh
git clone https://github.com/carrotciallo/poki-for-linux.git
cd poki-for-linux
./bin/setup
```

`./bin/setup` compila el puente nativo y las herramientas de idevice, instala las dependencias npm y verifica que los binarios precompilados de sharp estén realmente presentes (en algunas configuraciones npm de Linux se omiten en silencio; setup lo repara).

Prepara el iPhone una sola vez, conectado y desbloqueado por USB:

```sh
node bin/prepare-device.mjs
```

Esto instala la imagen de disco de desarrollador mediante pymobiledevice3. Si la imagen ya está instalada, lo informa y no hace nada.

Inicia el servidor y déjalo en ejecución:

```sh
npm start
```

Desde el mismo directorio, registra el servidor MCP en tu cliente:

```sh
claude mcp add --scope user poki-mcp -- node "$PWD/src/stdio.mjs"   # Claude Code
codex mcp add poki-mcp -- node "$PWD/src/stdio.mjs"                 # Codex
```

El estado se guarda en `$XDG_STATE_HOME/PokiMCP` (por defecto `~/.local/state/PokiMCP`), salvo que se defina `POKI_MCP_STATE_DIR`.

## Prueba una tarea

```sh
node bin/call.mjs list_devices   # Encuentra el UUID de tu dispositivo
node bin/call.mjs list_elements '{"device":"<device UUID>"}'
```

> **Nota:** Poki controla tu iPhone real. Si le pides que envíe mensajes o haga pagos, lo hará. El contenido de la pantalla se envía al servicio de modelo que hayas configurado. Las herramientas sensibles piden aprobación antes: en Linux el diálogo de aprobación usa `zenity`, y sin zenity las herramientas sensibles devuelven un error explícito en lugar de continuar en silencio.

**Poki Agent.** El agente de terminal independiente funciona igual en Linux (`npm ci --prefix agent` y luego `poki login`); consulta [Ejecutar Poki Agent](agent/README.md). La única excepción es el canal de iMessage: necesita un servidor BlueBubbles, que a su vez requiere un Mac. Telegram funciona.

**Conexión inalámbrica.** El uso remoto mediante Tailscale (RemotePairing) está integrado en el puente y es independiente de la plataforma; los pasos de [remote-iphone.md](docs/remote-iphone.md) se aplican igual, con el directorio de estado de este repositorio. La ruta de Tailscale en sí aún no está verificada en un anfitrión Linux, pero el mismo mecanismo RemotePairing sí se ha verificado aquí a través de la red local, usando la dirección LAN del iPhone en lugar de una de Tailscale; a partir de ahí, en casa ya no hace falta el cable.

## Verificado en hardware real

Anfitrión: Arch Linux (x86_64), usbmuxd 1.1.1, Node.js 24.22, pymobiledevice3 11.50. Dispositivo: iPhone18,4 con iOS 27.0.1, conexión USB.

| Comprobación | Resultado |
| --- | --- |
| Emparejamiento y comunicación lockdown vía usbmuxd | funciona |
| DeveloperDiskImage vía pymobiledevice3 | instalada, incluida una reinstalación completa con `bin/prepare-device.mjs` tras desinstalarla |
| `list_elements` (árbol de accesibilidad) | funciona |
| `take_screenshot` (captura del dispositivo sin pérdidas) | funciona |
| `tap`, `swipe`, `press_key` | funciona |
| `type_text` (entrada de texto verificada; el portapapeles anterior se restaura después) | funciona |
| Diálogo de aprobación vía `zenity` | funciona |
| Control inalámbrico por la LAN (RemotePairing a la IP local) | funciona |
| `npm test` | cada archivo de pruebas pasa por separado; consulta la nota en [LINUX.md](LINUX.md) |

## Limitaciones conocidas

- El asistente OCR es un binario de Vision de macOS, así que en Linux la comprobación de indicaciones en capturas informa `unavailable` y la observación se apoya en el árbol de accesibilidad (el mismo recurso que usa el proyecto original cuando falta el asistente).
- `get_clipboard` no puede clasificar los tipos de imagen del portapapeles en Linux.
- Algunas herramientas de dispositivo físico llaman a `xcrun devicectl` en el proyecto original (`open_url`, `terminate_app`, acceso a registros de fallos, exportación de contenedores de apps) y fallan en Linux hasta que se adapten a equivalentes de idevice-tools.
- Como en el proyecto original, los dispositivos físicos solo admiten orientación vertical.

## Documentación

| Tema | Lectura |
| --- | --- |
| Notas de la adaptación a Linux (alcance, diferencias con el original) | [LINUX.md](LINUX.md) |
| Instalación y solución de problemas | [Guía de instalación](docs/install.md) |
| Poki Agent y canales | [Ejecutar Poki Agent](agent/README.md) |
| Referencia de herramientas MCP | [Comportamiento de las herramientas](docs/tools.md) |
| Variables de entorno y sesiones | [Configuración](docs/configuration.md) |
| Emparejamiento USB, Modo de desarrollador y Wi-Fi | [Configuración de un iPhone físico](docs/physical-iphone.md) |
| iPhone a través de Tailscale | [iPhone remoto](docs/remote-iphone.md) |
| Resultados de pruebas y límites conocidos | [Verificación](docs/verification.md) |
| Arquitectura del puente | [Puente nativo](native/NATIVE.md) |

Parte de la documentación bajo `docs/` se escribió para el anfitrión macOS; cuando entre en conflicto con este README o con LINUX.md en lo relativo a la instalación del anfitrión, prevalecen los documentos de Linux.

## Aviso legal

Poki es un proyecto independiente de código abierto. No está afiliado ni respaldado por Apple, OpenAI ni por los creadores de ninguna app o sitio web que opera. iPhone es una marca comercial de Apple Inc. Esta adaptación es una bifurcación no oficial y no está afiliada al autor original.

Automatizar apps y sitios web de terceros puede infringir sus términos de servicio. Decide por ti mismo si usar Poki con un servicio dado. Eres responsable de lo que haga en tu dispositivo y tus cuentas, incluidos los mensajes que envíe, los cambios que haga y cualquier restricción de cuenta que resulte de ello.

Poki se ofrece "tal cual", sin garantías, según lo establecido en la licencia AGPL-3.0.

## Licencia

Poki es código abierto bajo la [licencia AGPL-3.0-only](LICENSE).

Copyright (c) 2026 Joseph Zhou y colaboradores. Las dependencias de código abierto conservan sus respectivas licencias; consulta [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). Para contribuir, consulta [CONTRIBUTING.md](CONTRIBUTING.md).

## Agradecimientos

<p align="center">
  <img src="docs/images/acknowledgments.png" width="860" alt="Construido sobre código abierto: pi, idevice, idb, BlueBubbles, Hermes Agent, MCP SDK, sharp y FFmpeg"><br>
  <sub>Agradecimientos de código abierto: <a href="https://github.com/earendil-works/pi">pi</a> | <a href="https://github.com/jkcoxson/idevice">idevice</a> | <a href="https://github.com/facebook/idb">idb</a> | <a href="https://github.com/BlueBubblesApp/bluebubbles-server">BlueBubbles</a> | <a href="https://github.com/NousResearch/hermes-agent">Hermes Agent</a> | <a href="https://github.com/modelcontextprotocol/typescript-sdk">MCP SDK</a> | <a href="https://github.com/lovell/sharp">sharp</a> | <a href="https://github.com/FFmpeg/FFmpeg">FFmpeg</a></sub>
</p>
