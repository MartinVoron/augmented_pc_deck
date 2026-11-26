// Évite d'ouvrir une console en plus de la fenêtre sous Windows (en version finale).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    augmented_pc_deck_lib::run()
}
