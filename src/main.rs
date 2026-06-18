//! cosmic-caffeine — indicador tipo "cafeína" para el panel de COSMIC.
//!
//! Aparece en la Status Area (StatusNotifierItem) y evita que la pantalla se
//! apague / el equipo se suspenda por inactividad mientras está activo.
//!
//! El bloqueo se hace por D-Bus contra `org.freedesktop.ScreenSaver`
//! (implementado por `cosmic-idle`): `Inhibit` devuelve una cookie y
//! `UnInhibit(cookie)` la libera. Si el proceso muere, cosmic-idle libera
//! automáticamente la inhibición al cerrarse la conexión.

use std::sync::OnceLock;
use std::time::Duration;

use ksni::blocking::{Handle, TrayMethods};
use ksni::menu::{CheckmarkItem, StandardItem, SubMenu};
use ksni::{MenuItem, ToolTip};
use zbus::blocking::{Connection, Proxy};

const APP_ID: &str = "cosmic-caffeine";
const ICON_ACTIVE: &str = "io.github.diegoachury.CaffeineCosmic-active-symbolic";
const ICON_INACTIVE: &str = "io.github.diegoachury.CaffeineCosmic-inactive-symbolic";
const SS_DEST: &str = "org.freedesktop.ScreenSaver";
const SS_PATH: &str = "/org/freedesktop/ScreenSaver";
const SS_IFACE: &str = "org.freedesktop.ScreenSaver";

/// Handle global hacia el tray, para que los temporizadores (que corren en
/// otros hilos) puedan pedir un re-render y soltar la sesión al expirar.
static HANDLE: OnceLock<Handle<CaffeineTray>> = OnceLock::new();

struct CaffeineTray {
    conn: Connection,
    active: bool,
    cookie: Option<u32>,
    /// Etiqueta de la sesión actual ("Indefinida", "1 hora", ...).
    session_label: String,
    /// Se incrementa en cada cambio de estado; invalida temporizadores viejos.
    generation: u64,
}

impl CaffeineTray {
    fn new() -> zbus::Result<Self> {
        Ok(Self {
            conn: Connection::session()?,
            active: false,
            cookie: None,
            session_label: String::new(),
            generation: 0,
        })
    }

    fn proxy(&self) -> zbus::Result<Proxy<'_>> {
        Proxy::new(&self.conn, SS_DEST, SS_PATH, SS_IFACE)
    }

    fn do_inhibit(&self) -> zbus::Result<u32> {
        let proxy = self.proxy()?;
        proxy.call("Inhibit", &(APP_ID, "El usuario activó cosmic-caffeine"))
    }

    fn do_uninhibit(&self, cookie: u32) -> zbus::Result<()> {
        let proxy = self.proxy()?;
        let _: () = proxy.call("UnInhibit", &(cookie,))?;
        Ok(())
    }

    /// Activa una sesión. `dur = None` => indefinida.
    fn engage(&mut self, label: &str, dur: Option<Duration>) {
        // Suelta cualquier sesión previa antes de abrir otra.
        self.disengage();
        match self.do_inhibit() {
            Ok(cookie) => {
                self.cookie = Some(cookie);
                self.active = true;
                self.session_label = label.to_string();
                self.generation = self.generation.wrapping_add(1);
                if let Some(d) = dur {
                    arm_timer(self.generation, d);
                }
            }
            Err(e) => eprintln!("cosmic-caffeine: error al inhibir: {e}"),
        }
    }

    fn disengage(&mut self) {
        if let Some(cookie) = self.cookie.take() {
            if let Err(e) = self.do_uninhibit(cookie) {
                eprintln!("cosmic-caffeine: error al liberar: {e}");
            }
        }
        self.active = false;
        self.session_label.clear();
        // Invalida temporizadores pendientes de la sesión anterior.
        self.generation = self.generation.wrapping_add(1);
    }

    fn toggle_indefinite(&mut self) {
        if self.active {
            self.disengage();
        } else {
            self.engage("Indefinida", None);
        }
    }
}

impl ksni::Tray for CaffeineTray {
    fn id(&self) -> String {
        APP_ID.into()
    }

    fn title(&self) -> String {
        "Cafeína".into()
    }

    fn icon_name(&self) -> String {
        if self.active {
            ICON_ACTIVE.into()
        } else {
            ICON_INACTIVE.into()
        }
    }

    fn tool_tip(&self) -> ToolTip {
        let description = if self.active {
            format!("Activa — {}", self.session_label)
        } else {
            "Inactiva".to_string()
        };
        ToolTip {
            title: "Cafeína".into(),
            description,
            icon_name: self.icon_name(),
            icon_pixmap: Vec::new(),
        }
    }

    /// Clic izquierdo sobre el icono: alterna sesión indefinida.
    fn activate(&mut self, _x: i32, _y: i32) {
        self.toggle_indefinite();
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let header = if self.active {
            format!("● Activa — {}", self.session_label)
        } else {
            "○ Inactiva".to_string()
        };

        let timed = |label: &'static str, secs: u64| -> MenuItem<Self> {
            StandardItem {
                label: label.into(),
                activate: Box::new(move |t: &mut CaffeineTray| {
                    t.engage(label, Some(Duration::from_secs(secs)))
                }),
                ..Default::default()
            }
            .into()
        };

        vec![
            StandardItem {
                label: header,
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            CheckmarkItem {
                label: "Mantener activo (indefinido)".into(),
                checked: self.active && self.session_label == "Indefinida",
                activate: Box::new(|t: &mut CaffeineTray| t.toggle_indefinite()),
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Activar por tiempo".into(),
                submenu: vec![
                    timed("15 minutos", 15 * 60),
                    timed("30 minutos", 30 * 60),
                    timed("1 hora", 60 * 60),
                    timed("2 horas", 2 * 60 * 60),
                    timed("4 horas", 4 * 60 * 60),
                ],
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Desactivar".into(),
                enabled: self.active,
                activate: Box::new(|t: &mut CaffeineTray| t.disengage()),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Salir".into(),
                activate: Box::new(|t: &mut CaffeineTray| {
                    t.disengage();
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// ¿Estamos dentro de un sandbox de Flatpak?
fn in_flatpak() -> bool {
    std::path::Path::new("/.flatpak-info").exists()
}

/// Lanza un hilo que, tras `dur`, suelta la sesión si sigue siendo la misma.
fn arm_timer(generation: u64, dur: Duration) {
    std::thread::spawn(move || {
        std::thread::sleep(dur);
        if let Some(handle) = HANDLE.get() {
            handle.update(move |t: &mut CaffeineTray| {
                if t.generation == generation && t.active {
                    t.disengage();
                }
            });
        }
    });
}

fn main() {
    let tray = match CaffeineTray::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cosmic-caffeine: no se pudo conectar a D-Bus: {e}");
            std::process::exit(1);
        }
    };

    // Dentro de Flatpak no podemos reclamar el nombre well-known del SNI
    // (el PID del sandbox no coincide); ksni ofrece `disable_dbus_name` para eso.
    let spawn_result = if in_flatpak() {
        tray.disable_dbus_name(true).spawn()
    } else {
        tray.spawn()
    };

    match spawn_result {
        Ok(handle) => {
            let _ = HANDLE.set(handle);
        }
        Err(e) => {
            eprintln!("cosmic-caffeine: no se pudo registrar en la bandeja: {e}");
            std::process::exit(1);
        }
    }

    // El servicio del tray corre en segundo plano; mantenemos vivo el proceso.
    loop {
        std::thread::park();
    }
}
