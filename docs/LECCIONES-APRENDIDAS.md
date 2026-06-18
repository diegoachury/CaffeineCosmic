# Lecciones aprendidas y buenas prácticas

Este documento recoge **lo que ocurrió realmente** durante el desarrollo y la
publicación de *Caffeine for COSMIC*, los **errores concretos** que aparecieron y,
sobre todo, las **buenas prácticas para no repetirlos** si vuelves a empezar un
proyecto similar (un applet/app de COSMIC distribuido en Flathub).

> TL;DR: la mayoría de los errores se evitan tomando **3 decisiones correctas el
> día 1**: (1) App ID y nombre de repo idénticos y sin guiones, (2) stack 100 %
> Rust puro (sin libs de C), (3) runtime de Flatpak más reciente. El resto son
> detalles de entorno (SSH, identidad git, deps de herramientas) fáciles de
> preparar de antemano.

---

## 1. Cronología de lo ocurrido

1. **Punto de partida.** COSMIC en Pop!_OS (Wayland). El `caffeine` clásico es de
   GNOME/X11 y no aplica.
2. **Investigación del entorno** (clave para acertar): se descubrió que `cosmic-idle`
   implementa `org.freedesktop.ScreenSaver` (Inhibit/UnInhibit) y que
   `cosmic-applet-status-area` actúa como host de StatusNotifierItem (SNI).
3. **Decisión de diseño:** app de bandeja SNI en Rust (`ksni` + `zbus`) que inhibe
   la inactividad por D-Bus. Nada de GTK ni AppIndicator.
4. **Build + verificación local** (icono en bandeja, activar/desactivar, sesiones
   temporizadas).
5. **Consolidación del repo:** README, `LICENSE` (GPL-3.0), `.gitignore`, metadatos
   de `Cargo.toml`, git remote.
6. **Empaquetado Flatpak:** manifiesto, `cargo-sources.json` (build offline),
   `metainfo.xml`, `Makefile`, `.desktop` e iconos **nombrados por App ID**.
7. **Build con `flatpak-builder` + linters.**
8. **Ajustes finales:** renombrar el repo para que coincida con el App ID y subir el
   runtime a 25.08.
9. **PR a Flathub** ([flathub/flathub#9031](https://github.com/flathub/flathub/pull/9031)).

---

## 2. Errores encontrados → causa → solución → cómo evitarlo

| # | Error / síntoma | Causa raíz | Solución aplicada | Buena práctica (desde el inicio) |
|---|---|---|---|---|
| 1 | `The system library 'dbus-1' required by crate 'libdbus-sys' was not found` | `ksni 0.2` enlaza la lib C `libdbus`; faltaba `libdbus-1-dev` | Migrar a **`ksni 0.3`** con feature `blocking` (D-Bus puro vía `zbus`) | Elegir crates **Rust puro**; revisar dependencias transitivas a C (`-sys`) antes de adoptar |
| 2 | `ModuleNotFoundError: No module named 'tomlkit'` al generar `cargo-sources.json` | `flatpak-cargo-generator.py` necesita `aiohttp` **y `tomlkit`** (no `toml`) | `pip install aiohttp tomlkit` en un venv | Tener identificadas las deps del generador |
| 3 | App ID inválido con guion | Los App ID de Flatpak/D-Bus solo admiten `[A-Za-z0-9_]` por componente | App ID en CamelCase: `io.github.diegoachury.CaffeineCosmic` | Definir el App ID **y nombrar el repo igual** (sin guiones) el día 1 |
| 4 | `pkill -f` mató su propio shell (exit 144) | El patrón `-f` coincidía con la línea de comando del propio script | Usar `pkill -x cosmic-caffeine` (nombre exacto) | Matar por **nombre exacto** (`-x`) o por PID (`pgrep`+`kill`), nunca `-f` con una cadena que aparezca en el propio comando |
| 5 | `Identidad del autor desconocido` al hacer el primer commit | Repo nuevo sin `user.name`/`user.email` | `git config user.name/.email` (local al repo) | Configurar identidad git al crear el repo |
| 6 | `Host key verification failed` + `ssh_askpass: ... No such file or directory` | `github.com` no estaba en `known_hosts` y el entorno es no interactivo | `ssh-keyscan` **verificando la huella** contra la publicada por GitHub antes de añadirla | Pre-sembrar `known_hosts` con la host key **verificada** |
| 7 | Linter: `appid-url-not-reachable` (`github.com/diegoachury/caffeinecosmic` → 404) | El repo se llamaba `caffeine-cosmic` (con guion) ≠ App ID `CaffeineCosmic` | `gh repo rename CaffeineCosmic` (+ actualizar remoto y URLs) | **Repo == último componente del App ID** desde el principio |
| 8 | `desktop-file-validate`: "more than one main category" | `Categories=Utility;System;` (dos categorías principales) | Dejar `Categories=Utility;` | Una sola categoría principal en el `.desktop` |
| 9 | `flatpak --system install ... No remote refs found for 'flathub'` durante el build | `--install-deps-from=flathub` intentó instalar en *system*; el remoto `flathub` es de **usuario** | Construir con `--user` y los runtimes/SDK preinstalados en `--user` | Instalar runtime + SDK + extensión en `--user` y compilar con `--user` |
| 10 | Linter: `appstream-external-screenshot-url` / `appstream-screenshots-not-mirrored-in-ostree` | Las capturas externas deben estar **espejadas** en `dl.flathub.org`; localmente no se espejan | **No es un defecto:** Flathub espeja automáticamente al construir. Para validarlo en local: `--mirror-screenshots-url=https://dl.flathub.org/media` (requiere red en el `compose`) | Hostear la captura en una **URL pública alcanzable** (rama por defecto) antes del PR; saber que Flathub la espeja |
| 11 | Aviso: `runtime-update-available-to-...-25.08` | Manifiesto apuntaba a `Platform 24.08` | Subir a `runtime-version: '25.08'` (+ `rust-stable//25.08`) | Apuntar al runtime **más reciente** desde el inicio (evita EOL) |
| 12 | Riesgo de romper el build al renombrar | Un `sed` global de `caffeine-cosmic` habría cambiado el **nombre del módulo** Flatpak (del que depende `CARGO_HOME=/run/build/caffeine-cosmic/cargo`) | Reemplazar solo `usuario/caffeine-cosmic` (URLs) y dejar el nombre del módulo intacto | Cuidado con find/replace global; usar patrones con contexto (`usuario/`) |

---

## 3. Buenas prácticas "desde cero" (orden ideal)

Si empezaras hoy un applet de COSMIC para Flathub, este orden evita **todos** los
errores anteriores:

### Día 1 — decisiones que cuesta cambiar después
1. **Elige el App ID y el nombre del repo a la vez** y hazlos idénticos:
   `io.github.<usuario>.<NombreEnCamelCase>` ↔ `github.com/<usuario>/<NombreEnCamelCase>`.
   Sin guiones, sin minúsculas que difieran del repo.
2. **Stack Rust puro:** `ksni` (≥ 0.3, feature `blocking`) + `zbus`. Evita crates `-sys`
   que enlacen C (no necesitarás `libdbus-1-dev`, `pkg-config`, etc.).
3. **Runtime más reciente** de freedesktop (hoy `25.08`) en el manifiesto.

### Preparación del entorno (una vez)
4. `git config user.name` / `user.email` en el repo nuevo.
5. Añade la host key de GitHub a `known_hosts` **verificando la huella**:
   ```bash
   ssh-keyscan -t ed25519 github.com > /tmp/gh && ssh-keygen -lf /tmp/gh
   # compara con la huella oficial de GitHub y solo entonces:
   cat /tmp/gh >> ~/.ssh/known_hosts
   ```
6. Instala el toolchain de Flatpak en `--user`:
   ```bash
   flatpak install --user flathub org.flatpak.Builder \
     org.freedesktop.Platform//25.08 org.freedesktop.Sdk//25.08 \
     org.freedesktop.Sdk.Extension.rust-stable//25.08
   ```

### Empaquetado correcto
7. **Nombra por App ID** el `.desktop`, el `.metainfo.xml` y los iconos
   (`<appid>.svg`, `<appid>-*-symbolic.svg`). Una sola `Categories=` principal.
8. **Detecta el sandbox** en el código: si existe `/.flatpak-info`, arranca `ksni`
   con `disable_dbus_name(true)` (si no, el icono no aparece dentro de Flatpak).
9. **finish-args mínimos** (los revisores lo agradecen): solo
   `--talk-name=org.freedesktop.ScreenSaver` y `--talk-name=org.kde.StatusNotifierWatcher`.
10. **Build offline:** genera `cargo-sources.json` con `flatpak-cargo-generator.py`
    (`pip install aiohttp tomlkit`) y mantén el nombre del **módulo** del manifiesto
    coherente con `CARGO_HOME=/run/build/<modulo>/cargo`. Regenéralo cuando cambie
    `Cargo.lock`.
11. **Screenshot:** súbela al repo y referénciala por URL pública en el `metainfo.xml`
    *antes* de abrir el PR (Flathub la espeja sola).

### Validación local antes del PR (== lo que hará la CI)
12. Compila e instala con `--user`:
    ```bash
    flatpak run org.flatpak.Builder --user --force-clean --install \
      build-dir build-aux/<appid>.yml
    ```
13. Pasa los linters y **verifica el icono en la bandeja** y la inhibición:
    ```bash
    flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest build-aux/<appid>.yml
    flatpak run --command=flatpak-builder-lint org.flatpak.Builder builddir build-dir
    ```
    El único hallazgo aceptable es el de **espejado de capturas** (lo hace Flathub).

### Release y envío
14. `git tag -a vX.Y.Z` → push del tag → fija `tag` **y** `commit` en el manifiesto.
15. Fork de `flathub/flathub`, rama `new-pr`, manifiesto + `cargo-sources.json` en la
    raíz, PR contra `new-pr`. Tras el merge, verifica el App ID en flathub.org (login
    con GitHub) para el sello *verified*.

---

## 4. Reglas de oro

- **El nombre lo es todo:** App ID = nombre de repo, sin guiones, fijado el día 1.
- **Rust puro > bindings a C:** menos dependencias de sistema = builds reproducibles.
- **`--user` en todo Flatpak** en una máquina de desarrollo personal.
- **Patrones, no `sed` ciego:** al renombrar, no toques el nombre del módulo Flatpak.
- **El linter local ≈ la CI**, salvo el espejado de capturas, que es responsabilidad
  de Flathub.
- **Verifica las host keys** antes de confiar en ellas.

Ver también [`DEPLOYMENT.md`](DEPLOYMENT.md) para el procedimiento detallado paso a paso.
