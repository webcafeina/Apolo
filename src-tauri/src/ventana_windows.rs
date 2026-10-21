//! La ventana en Windows (ADR 0027): sin marco, con Mica en Windows 11, y con
//! el menú de Snap sobre el botón de maximizar que dibuja la interfaz.
//!
//! La ventana se crea aquí y no desde `tauri.conf.json` (`tauri.windows.conf.json`
//! la deja en `"create": false`) por dos razones. La transparencia se decide al
//! crearla, y solo se quiere en Windows 11: en Windows 10 no hay Mica y la
//! ventana transparente se quedaría sin fondo. Y Tauri descarta el error de
//! `windowEffects`, así que no se sabría si Mica se puso; `window_vibrancy` lo
//! devuelve, y la interfaz pinta el fondo opaco si no (`data-vidrio="no"`).
//!
//! **Snap.** Windows enseña el menú de Snap cuando la ventana contesta
//! `HTMAXBUTTON` a `WM_NCHITTEST` sobre el punto del ratón, pero ahí la que
//! contesta es la ventana hija de WebView2, que no sabe hacerlo. La única
//! salida, la de todos los complementos de Tauri que lo resuelven, es otra
//! ventana hija, invisible (no pinta: pincel nulo), justo encima del botón
//! «Maximizar» de la barra de título, que contesta `HTMAXBUTTON`. Como se queda
//! con el ratón en ese recuadro, avisa a la interfaz de cuándo está encima o
//! pulsada (para que el botón se ilumine) y maximiza ella misma al hacer clic.
//! La técnica y sus trampas están en el estudio de Zbrooklyn/tauri-snap-layouts
//! y en el `snap.rs` de tauri-plugin-frame (los dos MIT); esto es una versión
//! propia y más corta, para una sola ventana.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

use tauri::{Emitter, WebviewWindow, WebviewWindowBuilder};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Dwm::{DWMWA_WINDOW_CORNER_PREFERENCE, DwmSetWindowAttribute};
use windows_sys::Win32::Graphics::Gdi::{GetStockObject, HBRUSH, NULL_BRUSH};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, HTMAXBUTTON, HWND_TOP, IsZoomed,
    RegisterClassExW, SW_MAXIMIZE, SW_RESTORE, SWP_ASYNCWINDOWPOS, SWP_SHOWWINDOW, SetWindowPos,
    ShowWindow, WM_DPICHANGED, WM_NCHITTEST, WM_NCLBUTTONDOWN, WM_NCLBUTTONUP, WM_NCMOUSELEAVE,
    WM_NCMOUSEMOVE, WM_SIZE, WNDCLASSEXW, WS_CHILD, WS_CLIPSIBLINGS, WS_OVERLAPPED, WS_VISIBLE,
};

/// Las medidas de la barra de título, en píxeles de 96 ppp: las mismas que
/// `.barra-titulo` y `.botones-ventana button` en frontend/src/sistema-windows.css.
/// Si cambian allí, aquí también, o la ventana de Snap no cae sobre el botón.
const ALTO_BARRA: u32 = 48;
const ANCHO_BOTON: u32 = 46;
/// Botones a la derecha de «Maximizar»: «Cerrar».
const A_LA_DERECHA: u32 = 1;

/// El primer Windows 11. Antes no hay Mica.
const WINDOWS_11: u32 = 22000;

/// El evento que oye la barra de título: `"encima"`, `"pulsado"` o `"fuera"`.
pub const EVENTO_MAXIMIZAR: &str = "ventana:maximizar";

static MICA: OnceLock<bool> = OnceLock::new();
static VENTANA: OnceLock<WebviewWindow> = OnceLock::new();
static PADRE: AtomicIsize = AtomicIsize::new(0);
static SUPERPUESTA: AtomicIsize = AtomicIsize::new(0);
static ENCIMA: AtomicBool = AtomicBool::new(false);
static PULSADO: AtomicBool = AtomicBool::new(false);

/// Si Mica se puso: la orden `plataforma` lo cuenta como vidrio.
pub fn hay_mica() -> bool {
    MICA.get().copied().unwrap_or(false)
}

/// Crea la ventana principal con la configuración de `tauri.windows.conf.json`.
pub fn crear(app: &tauri::App) -> tauri::Result<()> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|v| v.label == "principal")
        .expect("la ventana principal está en tauri.windows.conf.json")
        .clone();
    let once = windows_version::OsVersion::current().build >= WINDOWS_11;
    let ventana = WebviewWindowBuilder::from_config(app.handle(), &config)?
        .transparent(once)
        .build()?;
    // `None`: Mica sigue el tema del sistema, como la barra de título.
    let mica = once && window_vibrancy::apply_mica(&ventana, None).is_ok();
    let _ = MICA.set(mica);

    let hwnd = ventana.hwnd()?.0 as HWND;
    if once {
        // Sin marco, Windows 11 deja las esquinas a su criterio: se piden
        // redondeadas, como las de cualquier ventana (DWMWCP_ROUND).
        let redondas: u32 = 2;
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                (&redondas as *const u32).cast(),
                size_of::<u32>() as u32,
            );
        }
    }
    let _ = VENTANA.set(ventana.clone());
    // Las ventanas Win32 son del hilo que las crea: el de la interfaz. (Un
    // HWND no se puede mandar a otro hilo; su número sí.)
    let numero = hwnd as isize;
    ventana.run_on_main_thread(move || unsafe { poner_snap(numero as HWND) })?;
    Ok(())
}

unsafe fn poner_snap(padre: HWND) {
    let clase: Vec<u16> = "ApoloSnap\0".encode_utf16().collect();
    let instancia = unsafe { GetModuleHandleW(std::ptr::null()) };
    let registro = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(superpuesta),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instancia,
        hIcon: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        // Invisible porque no pinta nunca, no por ser transparente:
        // WS_EX_LAYERED le quitaría el ratón.
        hbrBackground: unsafe { GetStockObject(NULL_BRUSH) } as HBRUSH,
        lpszMenuName: std::ptr::null(),
        lpszClassName: clase.as_ptr(),
        hIconSm: std::ptr::null_mut(),
    };
    unsafe { RegisterClassExW(&registro) };
    // WS_CLIPSIBLINGS es lo que hace que gane a WebView2, su hermana.
    let hija = unsafe {
        CreateWindowExW(
            0,
            clase.as_ptr(),
            clase.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            padre,
            std::ptr::null_mut(),
            instancia,
            std::ptr::null(),
        )
    };
    if hija.is_null() {
        return;
    }
    PADRE.store(padre as isize, Ordering::Relaxed);
    SUPERPUESTA.store(hija as isize, Ordering::Relaxed);
    // Se recoloca al cambiar el tamaño o de pantalla (con otra escala).
    unsafe { SetWindowSubclass(padre, Some(padre_proc), 1, 0) };
    unsafe { colocar(padre) };
}

/// Encima del segundo botón por la derecha, a la escala de la pantalla.
unsafe fn colocar(padre: HWND) {
    let hija = SUPERPUESTA.load(Ordering::Relaxed) as HWND;
    let mut r = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if hija.is_null() || unsafe { GetClientRect(padre, &mut r) } == 0 {
        return;
    }
    let ppp = unsafe { GetDpiForWindow(padre) };
    let escala = |v: u32| ((v * ppp + 48) / 96) as i32;
    let ancho = escala(ANCHO_BOTON);
    unsafe {
        SetWindowPos(
            hija,
            HWND_TOP,
            r.right - ancho * (A_LA_DERECHA as i32 + 1),
            0,
            ancho,
            escala(ALTO_BARRA),
            SWP_ASYNCWINDOWPOS | SWP_SHOWWINDOW,
        )
    };
}

fn avisar(estado: &'static str) {
    if let Some(v) = VENTANA.get() {
        let _ = v.emit(EVENTO_MAXIMIZAR, estado);
    }
}

unsafe extern "system" fn padre_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _datos: usize,
) -> LRESULT {
    if msg == WM_SIZE || msg == WM_DPICHANGED {
        unsafe { colocar(hwnd) };
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

unsafe extern "system" fn superpuesta(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        // Lo que hace salir el menú de Snap.
        WM_NCHITTEST => return HTMAXBUTTON as LRESULT,
        WM_NCMOUSEMOVE => {
            if !ENCIMA.swap(true, Ordering::Relaxed) {
                avisar("encima");
            }
            // Para enterarse de cuándo se va (WM_NCMOUSELEAVE).
            let mut seguir = TRACKMOUSEEVENT {
                cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE | TME_NONCLIENT,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            unsafe { TrackMouseEvent(&mut seguir) };
            return 0;
        }
        WM_NCMOUSELEAVE => {
            ENCIMA.store(false, Ordering::Relaxed);
            PULSADO.store(false, Ordering::Relaxed);
            avisar("fuera");
            return 0;
        }
        WM_NCLBUTTONDOWN => {
            PULSADO.store(true, Ordering::Relaxed);
            avisar("pulsado");
            return 0;
        }
        WM_NCLBUTTONUP => {
            if PULSADO.swap(false, Ordering::Relaxed) {
                let padre = PADRE.load(Ordering::Relaxed) as HWND;
                let maximizada = unsafe { IsZoomed(padre) } != 0;
                unsafe { ShowWindow(padre, if maximizada { SW_RESTORE } else { SW_MAXIMIZE }) };
            }
            avisar(if ENCIMA.load(Ordering::Relaxed) {
                "encima"
            } else {
                "fuera"
            });
            return 0;
        }
        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
