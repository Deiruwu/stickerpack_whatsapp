# StickerPack

Generador de paquetes de stickers para WhatsApp desde PC, desarrollado en Rust.

StickerPack permite convertir imágenes y GIFs en stickers de 512 × 512 píxeles, organizarlos en un paquete y exportarlos como un archivo ZIP listo para importarse en WhatsApp mediante un código QR

El proyecto **no publica ni sube automáticamente los stickers a WhatsApp**. Se encarga únicamente de generar el paquete.

## Características

- **Configuración de escalado:** permite elegir la dimensión base para ajustar las imágenes.
- **Arreglo de frames:** los frames de los gifs estaban desfasados 

## Alertas

- El proyecto fue probado en Linux, no sé si funcione en Windows
- El proyecto admite imágenes separas o una carpeta completa

## Uso

```bash
stickerpack <ARCHIVOS...> --name <NOMBRE> --author <AUTOR> --base <DIMENSION>
```

### Ejemplo
```bash
stickerpack fotos/ gif1.gif img.png \
    --name "Mi pack" \
    --author "Dei" \
    --base height
```

También puedes proporcionar archivos individuales:

```bash
stickerpack sticker1.png sticker2.gif sticker3.jpg \
    --name "Mi pack" \
    --author "Dei" \
    --base height
```

### Parámetros

| Parámetro  | Descripción                                           |
|------------|-------------------------------------------------------|
| `inputs`   | Archivos y directorios de entrada.                    |
| `--name`   | Nombre del paquete.                                   |
| `--author` | Autor del paquete.                                    |
| `--base`   | Dimensión utilizada como referencia para el escalado. |
