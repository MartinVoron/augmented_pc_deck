//! Processus : usage du processeur, de la mémoire et des disques.
//!
//! Les mesures viennent de la crate `sysinfo`. Un seul [`ProcessMonitor`] est partagé
//! par toutes les tuiles : l'usage du processeur se calcule entre deux appels, donc
//! l'objet `System` doit être gardé d'une mesure à l'autre.

use std::sync::Mutex;

use serde::Serialize;
use sysinfo::{Disks, System};
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
