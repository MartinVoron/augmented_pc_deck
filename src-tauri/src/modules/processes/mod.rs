//! Processus : usage du processeur, de la mémoire et des disques, liste des processus
//! et arrêt d'un processus.
//!
//! Les mesures viennent de la crate `sysinfo`. Un seul [`ProcessMonitor`] est partagé
//! par toutes les tuiles : l'usage du processeur se calcule entre deux appels, donc
//! l'objet `System` doit être gardé d'une mesure à l'autre.

use std::sync::Mutex;

use serde::Serialize;
use sysinfo::{Disks, Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::State;

use crate::error::{AppError, AppResult};

/// Mesures partagées, rafraîchies à chaque appel de l'interface.
pub struct ProcessMonitor {
    system: Mutex<System>,
    disks: Mutex<Disks>,
}

impl Default for ProcessMonitor {
    fn default() -> Self {
        let mut system = System::new();
        // Première mesure : la suivante donnera un usage du processeur exact.
        system.refresh_cpu_usage();
        Self {
            system: Mutex::new(system),
            disks: Mutex::new(Disks::new_with_refreshed_list()),
        }
    }
}

/// Usage d'un disque, en octets.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskUsage {
    /// Lettre du lecteur (`C:`) ou point de montage.
    pub name: String,
    pub used: u64,
    pub total: u64,
}

/// Usage global du PC.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    /// Usage du processeur, en pourcentage (0 à 100).
    pub cpu: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disks: Vec<DiskUsage>,
}

/// Nom affiché d'un disque : `C:\` devient `C:`.
fn disk_name(mount_point: &str) -> String {
    let trimmed = mount_point.trim_end_matches(['\\', '/']);
    if trimmed.is_empty() {
        mount_point.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Systèmes de fichiers virtuels ou en lecture seule (Linux), sans intérêt pour la tuile :
/// paquets Snap (`squashfs`), mémoire (`tmpfs`), conteneurs (`overlay`)…
const VIRTUAL_FILE_SYSTEMS: [&str; 8] = [
    "squashfs",
    "tmpfs",
    "devtmpfs",
    "ramfs",
    "overlay",
    "efivarfs",
    "iso9660",
    "fuse.snapfuse",
];

/// Dossiers de montage réservés au système (Linux) : démarrage, Snap, Docker.
const SYSTEM_MOUNTS: [&str; 5] = [
    "/boot",
    "/snap",
    "/var/snap",
    "/var/lib/docker",
    "/run/snapd",
];

/// Indique si un disque intéresse l'utilisateur. Sous Windows, tous les lecteurs le sont.
fn is_user_disk(mount: &str, file_system: &str) -> bool {
    let file_system = file_system.to_ascii_lowercase();
    if VIRTUAL_FILE_SYSTEMS.contains(&file_system.as_str()) {
        return false;
    }
    !SYSTEM_MOUNTS.iter().any(|system| {
        mount == *system
            || mount
                .strip_prefix(system)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

/// Garde les disques utiles : vrai système de fichiers, taille connue, un seul par point
/// de montage, triés par nom. Chaque disque est donné par (montage, système de fichiers,
/// taille, espace libre).
fn collect_disks<'a>(disks: impl Iterator<Item = (&'a str, &'a str, u64, u64)>) -> Vec<DiskUsage> {
    let mut result: Vec<DiskUsage> = Vec::new();
    for (mount, file_system, total, available) in disks {
        let name = disk_name(mount);
        if total == 0 || !is_user_disk(mount, file_system) || result.iter().any(|d| d.name == name)
        {
            continue;
        }
        result.push(DiskUsage {
            name,
            used: total.saturating_sub(available),
            total,
        });
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

fn poisoned() -> AppError {
    AppError::Other("les mesures du système sont indisponibles".into())
}

/// Mesure l'usage du processeur, de la mémoire et des disques.
/// L'interface l'appelle toutes les 2 secondes : seules ces valeurs sont rafraîchies,
/// pas la liste des processus, pour ne pas charger le PC.
#[tauri::command]
pub async fn processes_usage(monitor: State<'_, ProcessMonitor>) -> AppResult<Usage> {
    let (cpu, memory_used, memory_total) = {
        let mut system = monitor.system.lock().map_err(|_| poisoned())?;
        system.refresh_cpu_usage();
        system.refresh_memory();
        (
            system.global_cpu_usage().clamp(0.0, 100.0),
            system.used_memory(),
            system.total_memory(),
        )
    };
    let disks = {
        let mut disks = monitor.disks.lock().map_err(|_| poisoned())?;
        // `true` : oublie les disques débranchés (clé USB retirée…).
        disks.refresh(true);
        let list = disks.list().iter().map(|d| {
            (
                d.mount_point().to_str().unwrap_or_default(),
                d.file_system().to_str().unwrap_or_default(),
                d.total_space(),
                d.available_space(),
            )
        });
        collect_disks(list)
    };
    Ok(Usage {
        cpu,
        memory_used,
        memory_total,
        disks,
    })
}

/// Un processus en cours d'exécution.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    /// Part du processeur utilisée, en pourcentage de toute la machine (comme le
    /// Gestionnaire des tâches), et non d'un seul cœur.
    pub cpu: f32,
    /// Mémoire utilisée, en octets.
    pub memory: u64,
}

/// Ramène l'usage d'un processus (somme sur tous les cœurs) à un pourcentage de la machine.
fn machine_percent(cpu: f32, cores: usize) -> f32 {
    (cpu / cores.max(1) as f32).clamp(0.0, 100.0)
}

/// Liste les processus en cours. La liste n'est mesurée que quand l'interface l'affiche.
#[tauri::command]
pub async fn processes_list(monitor: State<'_, ProcessMonitor>) -> AppResult<Vec<ProcessInfo>> {
    let mut system = monitor.system.lock().map_err(|_| poisoned())?;
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cpu().with_memory(),
    );
    let cores = system.cpus().len();
    let mut list: Vec<ProcessInfo> = system
        .processes()
        .values()
        // Le processus « inactif » du système (pid 0) n'est pas un vrai programme.
        // Sous Linux, `sysinfo` donne aussi les fils d'exécution (threads) et les tâches
        // du noyau : on ne garde que les vrais programmes.
        .filter(|p| p.pid().as_u32() != 0 && p.thread_kind().is_none())
        .map(|p| ProcessInfo {
            pid: p.pid().as_u32(),
            name: p.name().to_string_lossy().into_owned(),
            cpu: machine_percent(p.cpu_usage(), cores),
            memory: p.memory(),
        })
        .collect();
    list.sort_by_key(|p| std::cmp::Reverse(p.memory));
    Ok(list)
}

/// Arrête un processus. `name` est vérifié pour ne pas arrêter un autre programme
/// qui aurait repris le même numéro entre-temps.
#[tauri::command]
pub async fn processes_kill(
    monitor: State<'_, ProcessMonitor>,
    pid: u32,
    name: String,
) -> AppResult<()> {
    if pid == std::process::id() {
        return Err(AppError::InvalidInput(
            "Augmented PC Deck ne peut pas s'arrêter lui-même ici : utilisez Quitter".into(),
        ));
    }
    let mut system = monitor.system.lock().map_err(|_| poisoned())?;
    let pid = Pid::from_u32(pid);
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing(),
    );
    let process = system
        .process(pid)
        .filter(|p| p.name().to_string_lossy() == name)
        .ok_or_else(|| AppError::NotFound(format!("Le processus « {name} »")))?;
    if process.kill() {
        log::info!("processus arrêté : {name} ({pid})");
        Ok(())
    } else {
        let hint = if cfg!(windows) {
            "Windows refuse d'arrêter « {name} ». Il faut peut-être lancer l'application en administrateur"
        } else {
            "le système refuse d'arrêter « {name} » : il appartient sans doute à un autre utilisateur"
        };
        Err(AppError::PermissionDenied(hint.replace("{name}", &name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_disks_by_drive_letter() {
        assert_eq!(disk_name(r"C:\"), "C:");
        assert_eq!(disk_name("/home"), "/home");
        assert_eq!(disk_name("/"), "/");
    }

    #[test]
    fn keeps_one_entry_per_disk_with_a_size() {
        let disks = collect_disks(
            [
                (r"D:\", "NTFS", 1000, 400),
                (r"C:\", "NTFS", 500, 100),
                (r"C:\", "NTFS", 500, 100),
                (r"E:\", "", 0, 0),
            ]
            .into_iter(),
        );
        assert_eq!(
            disks,
            vec![
                DiskUsage {
                    name: "C:".into(),
                    used: 400,
                    total: 500
                },
                DiskUsage {
                    name: "D:".into(),
                    used: 600,
                    total: 1000
                },
            ]
        );
    }

    #[test]
    fn hides_virtual_and_system_disks_on_linux() {
        assert!(is_user_disk("/", "ext4"));
        assert!(is_user_disk("/home", "btrfs"));
        assert!(is_user_disk("/media/moi/CLE", "vfat"));
        assert!(is_user_disk("/bootcamp", "ntfs"));
        assert!(!is_user_disk("/snap/firefox/1234", "squashfs"));
        assert!(!is_user_disk("/run/user/1000", "tmpfs"));
        assert!(!is_user_disk("/boot/efi", "vfat"));
        assert!(!is_user_disk("/boot", "ext4"));
        assert!(!is_user_disk("/var/lib/docker/overlay2/abc", "ext4"));
    }

    #[test]
    fn converts_process_cpu_to_machine_percent() {
        assert_eq!(machine_percent(200.0, 8), 25.0);
        assert_eq!(machine_percent(50.0, 0), 50.0);
        assert_eq!(machine_percent(900.0, 4), 100.0);
    }

    #[test]
    fn measures_this_computer() {
        let monitor = ProcessMonitor::default();
        let usage = {
            let mut system = monitor.system.lock().unwrap();
            system.refresh_memory();
            system.total_memory()
        };
        assert!(usage > 0);
    }
}
