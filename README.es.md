<p align="center"><img src="src-tauri/icons/icon.svg" width="96" alt=""></p>

<h1 align="center">DaevaLens</h1>

<p align="center">Medidor de combate con varios overlays y análisis de combates para <b>AION 2</b> — Windows y Linux.<br>
<a href="README.md">Read in English</a></p>

---

DaevaLens te muestra lo que pasa en tus combates en pequeños overlays que colocas donde quieras encima del juego, y guarda cada combate para analizarlo después.

Solo **lee el tráfico de red que el juego recibe**. No toca el cliente, no lee su memoria y no automatiza nada.

## Hecho sobre A2Tools DPS Meter

Lo difícil de cualquier medidor de AION 2 —capturar el tráfico y descifrar el protocolo del juego— es obra de **[A2Tools DPS Meter](https://github.com/taengu/A2Tools-DPS-Meter)**, de **taengu** y colaboradores. DaevaLens usa su analizador, su captura y su almacén de combate **sin modificar**, como submódulo de git (`upstream/`): los arreglos del protocolo tras cada parche llegan actualizando el submódulo. Todo lo de encima —combates, overlays, rastreador, historial y análisis— es de DaevaLens.

Los dos proyectos son **GPL v3**. Si solo quieres un medidor de daño, usa A2Tools DPS Meter: es excelente.

## Qué hace

**Overlays** — cada uno es su propia ventana; se activan y se ajustan por separado, y se mueven y redimensionan en modo edición:

| Overlay | Muestra |
|---|---|
| DPS | Tu DPS y el de tu grupo, con una barra por jugador; tu fila siempre se ve |
| Objetivo | Vida del objetivo y tiempo estimado hasta que muera |
| Rastreador | Buffs en ti y debuffs que pones en tu objetivo: activo o no, cargas y tiempo activo |
| Habilidades | Tu daño por habilidad en el combate actual |
| Resumen | Tarjeta con los números del combate que acaba de terminar |
| Avisos | Aviso grande (y sonido) cuando cae un debuff que sigues o muere alguien |
| Estado | Conexión, ping y tiempo de combate |
| Próximos | Cuenta atrás de los eventos que sigues: grietas, asedios, jefes de mundo |

**Combates con sentido** — un combate empieza con tu primer golpe y termina tras unos segundos sin daño tuyo ni de tu grupo (configurable). Solo cuenta el daño a los enemigos que golpeas tú, así que los desconocidos de alrededor no inflan los números.

**Historial y análisis** — se guarda cada combate: DPS a lo largo del tiempo, daño por habilidad (crítico, espalda, mejor golpe), daño de invocaciones, tiempo activo de buffs y debuffs y todos los jugadores.

**Avisos de eventos** — grietas, asedio de artefactos, jefes tras el asedio, Vigilante Kaira, Señor guardián Nahma, festival Shugo, invasión dimensional y reinicios, con el calendario de los servidores globales (UTC o el reloj del servidor de tu región). Eliges cuáles seguir y te avisa unos minutos antes. Todas las horas se pueden editar —las fuentes de la comunidad no coinciden en algunas— y puedes añadir las tuyas.

**Listas por clase** — el rastreador se rellena solo para tu clase la primera vez que te ve (Spiritmaster: cargas de Unificación elemental, Corrosión, Maldición, Favor de Espíritu y Bendición ardiente). Desde el panel puedes añadir cualquier otro que veas.

**Arranca con el juego** — opcionalmente se abre al iniciar sesión en segundo plano; los overlays aparecen al abrir AION 2 y se esconden al cerrarlo. En Hyprland, además, se colocan en el escritorio del juego.

## Instalación

### Windows

1. Instala **[Npcap](https://npcap.com/#download)** (gratuito; es el mismo controlador que usa Wireshark).
2. Descarga el instalador desde [Releases](../../releases) y ejecútalo.

### Linux

El juego va con Proton; DaevaLens funciona de forma nativa a su lado.

- **Debian / Ubuntu / Mint / Pop!_OS:** `sudo apt install ./DaevaLens_*.deb`
- **Fedora / openSUSE:** `sudo dnf install ./DaevaLens-*.rpm`
- **Arch / CachyOS / Manjaro / EndeavourOS:** `cd packaging/arch && makepkg -si`

Los paquetes dan al programa el único permiso que necesita para leer paquetes (`CAP_NET_RAW`), así que nunca se ejecuta como root. Si lo compilas tú:

```bash
sudo setcap cap_net_raw=ep /ruta/a/daevalens
```

El icono de la bandeja necesita `libayatana-appindicator`; sin ella DaevaLens funciona igual, solo que sin bandeja.

La configuración para Hyprland, Sway e i3 está en el [README en inglés](README.md#wayland-compositors).

## Atajos

Asígnalos a teclas en tu escritorio o usa el menú de la bandeja:

```text
daevalens --panel            abre el panel
daevalens --edit             activa o desactiva el modo edición
daevalens --toggle <widget>  muestra u oculta: dps target tracker skills summary alerts info events
daevalens --reset            reinicia el combate actual
```

## ¿Está permitido?

DaevaLens es pasivo: lee paquetes que el juego ya recibe, como cualquier herramienta de red. NCSOFT no lo respalda y usar herramientas de terceros es bajo tu responsabilidad. No uses nunca nada que juegue por ti.

## Licencia

GPL-3.0. AION 2 es una marca de NCSOFT; este proyecto no tiene relación con NCSOFT.
