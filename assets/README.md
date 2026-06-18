# assets

`screenshot.png` — captura del menú de Caffeine desplegado en el panel de COSMIC.

Es **obligatoria** para el catálogo de Flathub: la referencia el `metainfo.xml`
mediante la URL

```
https://raw.githubusercontent.com/diegoachury/CaffeineCosmic/main/assets/screenshot.png
```

por lo que debe existir en `main` antes de que la CI de Flathub valide el envío.
Para regenerarla, recorta una captura del panel con el menú abierto (p. ej. con
`ffmpeg -i captura.png -vf "crop=W:H:X:Y" assets/screenshot.png`).
