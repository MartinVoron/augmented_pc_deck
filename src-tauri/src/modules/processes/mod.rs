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

/// Garde les disques utiles : taille connue, un seul par point de montage, triés par nom.
fn collect_disks<'a>(disks: impl Iterator<Item = (&'a str, u64, u64)>) -> Vec<DiskUsage> {
    let mut result: Vec<DiskUsage> = Vec::new();
    for (mount, total, available) in disks {
        let name = disk_name(mount);
        if total == 0 || result.iter().any(|d| d.name == name) {
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
        .filter(|p| p.pid().as_u32() != 0)
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
        Err(AppError::PermissionDenied(format!(
            "Windows refuse d'arrêter « {name} ». Il faut peut-être lancer l'application en administrateur"
        )))
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
                (r"D:\", 1000, 400),
                (r"C:\", 500, 100),
                (r"C:\", 500, 100),
                (r"E:\", 0, 0),
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
