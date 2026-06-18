# Guía de despliegue en Flathub

Guía completa y autocontenida para publicar **Caffeine for COSMIC**
(`io.github.diegoachury.CaffeineCosmic`) en [Flathub](https://flathub.org). Una vez
publicado, la app aparece automáticamente en la **tienda COSMIC** (`cosmic-store`),
que es solo un frontend de Flatpak (remotos `flathub` y `cosmic`).

> Mantenedor: **Diego Achury** · GitHub `diegoachury` · repo
> `git@github.com:diegoachury/CaffeineCosmic.git`

---

## 1. Visión general

La app es un indicador de bandeja **StatusNotifierItem** (crate `ksni` 0.3, feature
`blocking`) que inhibe la inactividad por D-Bus contra `org.freedesktop.ScreenSaver`
(interfaz implementada por `cosmic-idle` en COSMIC). No es un applet del panel
COSMIC, sino una app autónoma — por eso encaja en el modelo Flatpak estándar.

Flujo de publicación:

```
  Repo upstream (GitHub: diegoachury/CaffeineCosmic)
        │  git tag vX.Y.Z  +  assets/screenshot.png en main
        ▼
  Manifiesto Flatpak (build-aux/io.github.diegoachury.CaffeineCosmic.yml)
   + cargo-sources.json (dependencias vendored para build sin red)
        │
        ▼
  flatpak-builder LOCAL  ──►  pruebas (icono en bandeja + inhibición)
        │  validaciones (appstreamcli, desktop-file-validate, flatpak-builder-lint)
        ▼
  PR a flathub/flathub (rama new-pr)  ──►  build-bot construye en CI
        │  revisión + merge
        ▼
  Publicado en Flathub  ──►  visible en cosmic-store
```

**Reparto del trabajo:** las piezas técnicas (manifiesto, metainfo, Makefile,
`cargo-sources.json`, ajuste de sandbox) ya están en el repo. Quedan tareas del
mantenedor: instalar herramientas, hacer público el repo, etiquetar el release,
probar localmente y abrir el PR.

---

## 2. Prerrequisitos

Todo se puede instalar **sin `sudo`** (Flatpak a nivel de usuario).

```bash
# Herramienta oficial de build + linters de Flathub (trae flatpak-builder dentro)
flatpak install --user flathub org.flatpak.Builder

# Runtime y SDK que usa el manifiesto
flatpak install --user flathub org.freedesktop.Platform//24.08
flatpak install --user flathub org.freedesktop.Sdk//24.08
flatpak install --user flathub org.freedesktop.Sdk.Extension.rust-stable//24.08
```

Validadores de AppStream/desktop (normalmente ya presentes en Pop!_OS; si no, vía
gestor del sistema):

```bash
appstreamcli --version          # validación del metainfo
desktop-file-validate --version # validación del .desktop  (paquete: desktop-file-utils)
```

Otros requisitos:

| Requisito | Detalle |
|---|---|
| Cuenta en **flathub.org** | Inicia sesión **con GitHub** (necesario para verificar el App ID `io.github.*`). |
| Repo público en GitHub | `CaffeineCosmic` debe ser público antes de abrir el PR. |
| `git`, `python3` con `venv` | Para regenerar `cargo-sources.json` (ver §4). |

> **Nota:** `flatpak-builder` también existe como paquete del sistema, pero usar
> `org.flatpak.Builder` (Flatpak) garantiza la misma versión que la CI de Flathub e
> incluye `flatpak-builder-lint`.

---

## 3. Anatomía del repositorio

| Archivo | Rol |
|---|---|
| `build-aux/io.github.diegoachury.CaffeineCosmic.yml` | **Manifiesto Flatpak.** Define runtime, SDK, `finish-args` (permisos de sandbox) y el módulo de build. |
| `build-aux/cargo-sources.json` | **Dependencias de Cargo vendored** para el build offline de Flathub (sin red). Generado con `flatpak-cargo-generator.py`. |
| `data/io.github.diegoachury.CaffeineCosmic.desktop` | Lanzador. `Exec=cosmic-caffeine`, `Icon=io.github.diegoachury.CaffeineCosmic`. |
| `data/io.github.diegoachury.CaffeineCosmic.metainfo.xml` | Metadatos AppStream (nombre, resumen, descripción, licencia, OARS, releases, screenshot). |
| `data/icons/hicolor/scalable/apps/io.github.diegoachury.CaffeineCosmic.svg` | Icono de la app (catálogo / lanzador). |
| `data/icons/hicolor/scalable/status/io.github.diegoachury.CaffeineCosmic-active-symbolic.svg` | Icono de bandeja, estado **activo**. |
| `data/icons/hicolor/scalable/status/io.github.diegoachury.CaffeineCosmic-inactive-symbolic.svg` | Icono de bandeja, estado **inactivo**. |
| `assets/screenshot.png` | Captura referenciada por el metainfo (obligatoria para Flathub). |
| `Makefile` | Instala binario + datos en `$(DESTDIR)$(PREFIX)`. El manifiesto invoca `make install PREFIX=/app`. |
| `Cargo.toml` / `Cargo.lock` | Crate Rust. `Cargo.lock` es la **fuente de verdad** de `cargo-sources.json`. |
| `install.sh` | Instalación local desde fuente (no usada por Flatpak). |

### Permisos de sandbox (`finish-args`)

El manifiesto pide lo mínimo:

```yaml
finish-args:
  - --talk-name=org.freedesktop.ScreenSaver       # inhibir inactividad (cosmic-idle)
  - --talk-name=org.kde.StatusNotifierWatcher      # registrar el icono en la bandeja
```

No hay `--own-name`: dentro de Flatpak la app detecta `/.flatpak-info` y llama a
`disable_dbus_name(true)`, por lo que registra el SNI con su nombre único en vez de
reclamar `org.kde.StatusNotifierItem-PID-ID` (que fallaría por el PID del sandbox).

---

## 4. Regenerar `cargo-sources.json`

Necesario **cada vez que cambie `Cargo.lock`** (nueva dependencia o versión). Si no,
el build offline de Flathub fallará al no encontrar el crate vendored.

```bash
cd ~/projects/cosmic-caffeine

# 1) Entorno aislado con las deps del generador
python3 -m venv /tmp/fcg-venv
/tmp/fcg-venv/bin/pip install -q aiohttp tomlkit

# 2) Descargar el generador oficial de flatpak-builder-tools
curl -fsSL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py \
  -o /tmp/flatpak-cargo-generator.py

# 3) Generar el archivo a partir de Cargo.lock
/tmp/fcg-venv/bin/python /tmp/flatpak-cargo-generator.py Cargo.lock -o build-aux/cargo-sources.json
```

### Cómo funciona (patrón *vendored*)

El generador produce un array de fuentes con `dest: cargo/vendor/<crate>` y una
entrada `inline` que escribe `cargo/config`:

```toml
[source.vendored-sources]
directory = "cargo/vendor"
[source.crates-io]
replace-with = "vendored-sources"
```

Por eso el manifiesto define `CARGO_HOME: /run/build/caffeine-cosmic/cargo` (donde
`caffeine-cosmic` es el **nombre del módulo**, que fija `/run/build/caffeine-cosmic`).
Cargo encuentra ahí su `config`, y el `directory = "cargo/vendor"` (relativo al
directorio de build) le da los crates sin acceder a la red.

> ⚠️ Si renombras el módulo en el manifiesto, **debes** actualizar `CARGO_HOME` en
> consecuencia. Mantenlo como `caffeine-cosmic`.

---

## 5. Build y prueba LOCAL con flatpak-builder

### 5.1 Probar antes de tener el tag publicado (fuente `type: dir`)

El manifiesto del repo apunta a un `tag` de GitHub (para Flathub). Para probar tus
cambios locales **antes** de etiquetar, crea una copia temporal del manifiesto con
una fuente que apunte a la carpeta de trabajo:

```bash
cd ~/projects/cosmic-caffeine
cp build-aux/io.github.diegoachury.CaffeineCosmic.yml /tmp/test.yml
```

Edita `/tmp/test.yml` y sustituye el bloque `sources:` por:

```yaml
    sources:
      - type: dir
        path: ../..            # raíz del repo, relativa a build-aux/ (donde vive cargo-sources.json)
      - cargo-sources.json
```

> Con `type: dir` el build usa tu árbol actual (incluido `Cargo.lock`), así que
> asegúrate de haber regenerado `cargo-sources.json` (§4) si tocaste dependencias.

### 5.2 Construir e instalar

```bash
cd ~/projects/cosmic-caffeine/build-aux

# Opción A: flatpak-builder del paquete org.flatpak.Builder
flatpak run org.flatpak.Builder --user --install --force-clean build-dir /tmp/test.yml

# Opción B: si tienes flatpak-builder del sistema
flatpak-builder --user --install --force-clean build-dir /tmp/test.yml
```

Para el build "de verdad" (el que usará Flathub) usa el manifiesto real, que requiere
que el `tag`/`commit` ya estén publicados (ver §7):

```bash
flatpak run org.flatpak.Builder --user --install --force-clean \
  build-dir io.github.diegoachury.CaffeineCosmic.yml
```

### 5.3 Ejecutar y verificar

```bash
flatpak run io.github.diegoachury.CaffeineCosmic
```

Comprobaciones manuales:

1. **El icono aparece en la bandeja** (Status Area del panel COSMIC).
2. **Clic izquierdo** alterna entre activo/inactivo (el icono cambia ☕ ⇄ ☕-tachado).
3. **La inhibición funciona**: con la sesión activa, verifica que se tomó la cookie:
   ```bash
   busctl --user call org.freedesktop.ScreenSaver /org/freedesktop/ScreenSaver \
     org.freedesktop.ScreenSaver Inhibit ss "test" "test"
   # (devuelve un cookie; UnInhibit u <cookie> para liberar)
   ```
   En la práctica: activa una sesión y confirma que la pantalla no se apaga con el
   tiempo de inactividad configurado en Ajustes de COSMIC.

### 5.4 Depuración: el icono NO aparece dentro del sandbox

Es el problema clásico de SNI + Flatpak. Pasos:

1. Confirma que el proceso arranca: `flatpak run io.github.diegoachury.CaffeineCosmic`
   y mira stderr (no debe quedarse en "no se pudo registrar en la bandeja").
2. Verifica que la app detecta el sandbox (debe existir `/.flatpak-info` dentro) y
   por tanto usa `disable_dbus_name(true)`.
3. Comprueba el watcher desde el host:
   ```bash
   busctl --user get-property org.kde.StatusNotifierWatcher \
     /StatusNotifierWatcher org.kde.StatusNotifierWatcher RegisteredStatusNotifierItems
   ```
   Debe listar un item nuevo cuando la app está corriendo.
4. Si aun así no aparece, **último recurso**: en el manifiesto sustituye los dos
   `--talk-name` por acceso completo al bus de sesión:
   ```yaml
   finish-args:
     - --socket=session-bus
   ```
   Es más permisivo (los revisores de Flathub prefieren `--talk-name`), así que úsalo
   solo si demuestras que es imprescindible, y documéntalo en el PR.

---

## 6. Validaciones obligatorias antes del PR

```bash
cd ~/projects/cosmic-caffeine

# AppStream
appstreamcli validate data/io.github.diegoachury.CaffeineCosmic.metainfo.xml

# Desktop entry
desktop-file-validate data/io.github.diegoachury.CaffeineCosmic.desktop

# Linters de Flathub (los mismos que la CI)
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
  manifest build-aux/io.github.diegoachury.CaffeineCosmic.yml
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
  appstream data/io.github.diegoachury.CaffeineCosmic.metainfo.xml
```

> **Sobre el aviso `screenshot-image-not-found`:** `appstreamcli` falla mientras la
> URL `https://raw.githubusercontent.com/diegoachury/CaffeineCosmic/main/assets/screenshot.png`
> no exista en la rama `main`. Es **esperado** hasta que subas el repo con
> `assets/screenshot.png`; tras el push, el aviso desaparece. El resto del metainfo
> ya es estructuralmente válido.

También puedes lintar el **build terminado**:

```bash
flatpak run --command=flatpak-builder-lint org.flatpak.Builder builddir build-dir
```

---

## 7. Preparar el release upstream

```bash
cd ~/projects/cosmic-caffeine

# 1) El repo debe ser PÚBLICO en GitHub (hazlo en Settings → General → Danger Zone, o al crearlo).

# 2) Sube todo a main (incluido assets/screenshot.png) y etiqueta el release
git push -u origin main
git tag v0.1.0
git push origin v0.1.0

# 3) Obtén el SHA exacto del commit del tag
git rev-parse v0.1.0
```

Copia ese SHA y **rellena el campo `commit:`** del manifiesto (ahora está a ceros con
un `TODO`):

```yaml
      - type: git
        url: https://github.com/diegoachury/CaffeineCosmic.git
        tag: v0.1.0
        commit: <SHA_DE_git_rev-parse_v0.1.0>
```

> Flathub **exige** `commit:` además de `tag:` (inmutabilidad del build). Sin el SHA
> correcto, la CI rechaza el PR.

---

## 8. Envío a Flathub

Flathub se gestiona por PRs al repo [`flathub/flathub`](https://github.com/flathub/flathub).

```bash
# 1) Haz fork de https://github.com/flathub/flathub  (botón Fork en GitHub)

# 2) Clona tu fork y crea la rama de envío DESDE la rama por defecto del repo
git clone git@github.com:diegoachury/flathub.git
cd flathub
git checkout -b new-pr           # nombre de rama recomendado por Flathub para nuevas apps
```

**Qué poner en el repo de envío** (en la raíz, no en `build-aux/`):

```bash
# El manifiesto, renombrado/colocado en la raíz con el nombre del App ID
cp ~/projects/cosmic-caffeine/build-aux/io.github.diegoachury.CaffeineCosmic.yml ./
# Las fuentes vendored de cargo, también en la raíz
cp ~/projects/cosmic-caffeine/build-aux/cargo-sources.json ./
```

Como ahora el manifiesto y `cargo-sources.json` están en el **mismo directorio**
(la raíz), la referencia `- cargo-sources.json` del bloque `sources` sigue siendo
correcta (es relativa al manifiesto). No hace falta tocar más rutas.

```bash
git add io.github.diegoachury.CaffeineCosmic.yml cargo-sources.json
git commit -m "Add io.github.diegoachury.CaffeineCosmic"
git push -u origin new-pr
```

Abre el **Pull Request** desde tu rama `new-pr` hacia `flathub/flathub:master`.

### El bot y la revisión

- El **build-bot** de Flathub construye el manifiesto en CI automáticamente al abrir
  el PR y en cada push. Si falla, corrige y vuelve a empujar.
- Comandos típicos en los comentarios del PR (los ejecuta el bot):
  - `bot, build io.github.diegoachury.CaffeineCosmic` — fuerza un build.
  - El bot publica un enlace para **instalar el build de prueba** y verificarlo antes
    del merge:
    ```bash
    flatpak install --user <repo-de-prueba-que-indica-el-bot> io.github.diegoachury.CaffeineCosmic
    ```
- Un revisor humano comprueba permisos de sandbox, metainfo e icono. Responde a sus
  comentarios; cuando aprueba, **hace merge** y la app entra en la cola de publicación.

Tras el merge, Flathub crea un repo dedicado `flathub/io.github.diegoachury.CaffeineCosmic`
del que saldrán las futuras actualizaciones.

---

## 9. Tras la publicación

```bash
# 1) Verifica el App ID como tuyo: en https://flathub.org inicia sesión con GitHub,
#    ve a tu app y pulsa "Verify" (al ser io.github.diegoachury.* se valida con tu cuenta).

# 2) Instala desde Flathub
flatpak install --user flathub io.github.diegoachury.CaffeineCosmic
flatpak run io.github.diegoachury.CaffeineCosmic
```

3. Abre **cosmic-store** y busca "Caffeine" — debe aparecer (cosmic-store lee el
   remoto `flathub`). La propagación al catálogo puede tardar unas horas.

---

## 10. Actualizaciones futuras

Para publicar una versión nueva (p. ej. `0.2.0`):

1. **Upstream** (repo `CaffeineCosmic`):
   ```bash
   # Sube la versión del crate
   #   Cargo.toml:  version = "0.2.0"
   # Añade una entrada al metainfo:
   #   <release version="0.2.0" date="AAAA-MM-DD"> ... </release>
   cargo build --release          # actualiza Cargo.lock si cambió algo
   # Si Cargo.lock cambió, regenera las fuentes vendored:
   #   (ver §4)  -> build-aux/cargo-sources.json
   git commit -am "Release 0.2.0"
   git push
   git tag v0.2.0 && git push origin v0.2.0
   git rev-parse v0.2.0           # nuevo SHA
   ```
2. **Repo de Flathub** (`flathub/io.github.diegoachury.CaffeineCosmic`):
   - Actualiza `tag:` y `commit:` en el manifiesto.
   - Si cambiaron dependencias, copia el nuevo `cargo-sources.json`.
   - Abre un PR de actualización; el bot construye y, tras merge, se publica.

---

## 11. Solución de problemas

| Síntoma | Causa probable | Solución |
|---|---|---|
| Build offline falla: `failed to get <crate>` / `no matching package` | Se añadió/actualizó una dependencia y `cargo-sources.json` quedó desfasado | Regenerar `cargo-sources.json` desde el `Cargo.lock` actual (§4). |
| `cargo` intenta acceder a la red durante el build | `CARGO_HOME` no apunta al `cargo/` vendored, o el nombre del módulo no es `caffeine-cosmic` | Verifica `env.CARGO_HOME: /run/build/caffeine-cosmic/cargo` y que el módulo se llame `caffeine-cosmic`. |
| El icono no aparece en la bandeja (en Flatpak) | Problema SNI en sandbox / nombre D-Bus | Confirma `disable_dbus_name(true)` (detección de `/.flatpak-info`); como último recurso `--socket=session-bus` (§5.4). |
| `appstreamcli`/CI: `screenshot-image-not-found` | El PNG aún no está en `main` o la URL es incorrecta | Sube `assets/screenshot.png` a `main`; comprueba que la URL `raw.githubusercontent.com/.../main/assets/screenshot.png` resuelve. |
| CI: `runtime is end-of-life` | `org.freedesktop.Platform//24.08` quedó obsoleto | Sube `runtime-version` a la versión vigente y vuelve a probar/lint. |
| CI: `appid-uses-code-hosting-domain` no verificado | App ID no coincide con tu cuenta o falta verificación | Asegura `io.github.diegoachury.*` y verifica en flathub.org con GitHub (§9). |
| `make install` no encuentra el binario | El paso `cargo build --release` no se ejecutó o falló | Revisa el orden de `build-commands`; el binario debe estar en `target/release/cosmic-caffeine`. |

---

## 12. Checklist final pre-PR

- [ ] Repo `CaffeineCosmic` **público** en GitHub.
- [ ] `assets/screenshot.png` presente en `main`.
- [ ] `Cargo.lock` commiteado y `cargo-sources.json` regenerado a partir de él.
- [ ] `git tag vX.Y.Z` creado y empujado; `commit:` del manifiesto = `git rev-parse vX.Y.Z`.
- [ ] `<release>` del metainfo coincide con la versión y tiene fecha.
- [ ] `appstreamcli validate` ✅ (sin errores; el screenshot resuelve tras el push).
- [ ] `desktop-file-validate` ✅.
- [ ] `flatpak-builder-lint manifest` y `... appstream` ✅.
- [ ] Build local con `flatpak-builder --install` ✅ y app probada:
  - [ ] icono aparece en la bandeja,
  - [ ] alterna activo/inactivo,
  - [ ] inhibición efectiva (la pantalla no se apaga).
- [ ] Fork de `flathub/flathub`, rama `new-pr`, manifiesto + `cargo-sources.json` en la raíz.
- [ ] PR abierto; build-bot en verde.
- [ ] App ID verificado en flathub.org tras el merge.
