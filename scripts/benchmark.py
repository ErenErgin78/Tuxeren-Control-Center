#!/usr/bin/env python3
"""
Benchmark Tool: Tuxedo Control Center vs Monster Control Center (Tuxeren)
Measures full process tree CPU and RSS Memory averages over a configurable time window
while the user actively navigates through tabs and UI pages.
"""

import sys
import os
import time
import json
import argparse
from typing import List, Dict, Any, Tuple
import psutil

RESULTS_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "benchmark_results.json")

def find_target_processes(target: str) -> Dict[str, List[psutil.Process]]:
    """
    Finds all running processes for the selected application.
    Returns a dictionary: {'daemon': [Process, ...], 'gui': [Process, ...]}
    """
    my_pid = os.getpid()
    gui_procs: List[psutil.Process] = []
    daemon_procs: List[psutil.Process] = []

    for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
        if proc.pid == my_pid:
            continue
        try:
            cmdline_list = proc.info.get('cmdline') or []
            cmdline = ' '.join(cmdline_list)
            name = proc.info.get('name') or ''

            if target == "tuxedo":
                # Tuxedo Daemon
                if 'data/service/tccd' in cmdline or name == 'tccd' or ('tccd' in cmdline and '--start' in cmdline):
                    daemon_procs.append(proc)
                # Tuxedo GUI Root & children
                elif 'tuxedo-control-center' in cmdline or name == 'tuxedo-control-center':
                    if '--type=' not in cmdline:
                        gui_procs.append(proc)
                        try:
                            for child in proc.children(recursive=True):
                                if child not in gui_procs:
                                    gui_procs.append(child)
                        except (psutil.NoSuchProcess, psutil.AccessDenied):
                            pass

            elif target == "monster":
                # Monster Daemon (txectrl daemon)
                if 'txectrl daemon' in cmdline or (name == 'txectrl' and 'daemon' in cmdline):
                    daemon_procs.append(proc)
                # Monster GUI Root (txectrl-gui) & WebKit child processes
                elif 'txectrl-gui' in cmdline or name == 'txectrl-gui':
                    gui_procs.append(proc)
                    try:
                        for child in proc.children(recursive=True):
                            if child not in gui_procs:
                                gui_procs.append(child)
                    except (psutil.NoSuchProcess, psutil.AccessDenied):
                        pass

        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue

    return {
        'daemon': list(set(daemon_procs)),
        'gui': list(set(gui_procs))
    }

def sample_resources(procs: List[psutil.Process]) -> Tuple[float, float]:
    """
    Returns (total_cpu_percent, total_rss_mb) for the given processes.
    """
    total_cpu = 0.0
    total_rss_bytes = 0
    active_procs = []

    for p in procs:
        try:
            if p.is_running() and p.status() != psutil.STATUS_ZOMBIE:
                total_cpu += p.cpu_percent(interval=None)
                total_rss_bytes += p.memory_info().rss
                active_procs.append(p)
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue

    rss_mb = total_rss_bytes / (1024 * 1024)
    return total_cpu, rss_mb

def run_benchmark(target: str, duration: int = 30, interval: float = 0.5) -> Dict[str, Any]:
    app_name = "TUXEDO Control Center (Electron + Node)" if target == "tuxedo" else "Monster Control Center / Tuxeren (Tauri + Rust)"
    print("\n" + "=" * 70)
    print(f"  BENCHMARK: {app_name}")
    print(f"  Test Süresi: {duration} saniye (Örnekleme aralığı: {interval}s)")
    print("=" * 70)

    # Initial check
    proc_dict = find_target_processes(target)
    all_procs = proc_dict['daemon'] + proc_dict['gui']

    if not all_procs:
        print(f"\n[!] HATA: '{target}' için çalışan herhangi bir süreç bulunamadı!")
        if target == "tuxedo":
            print("    Lütfen uygulamayı başlatın: 'tuxedo-control-center' ve servisi kontrol edin: 'systemctl status tccd'")
        else:
            print("    Lütfen uygulamayı başlatın: 'txectrl-gui' ve arka plan servisini açın: 'txectrl daemon'")
        sys.exit(1)

    print(f"\n[*] Tespit Edilen Süreçler:")
    if proc_dict['daemon']:
        print(f"    - Arka Plan Servisi ({len(proc_dict['daemon'])} süreç): PIDs {[p.pid for p in proc_dict['daemon']]}")
    else:
        print("    - [!] Arka plan servisi (daemon) çalışmıyor olarak görünüyor.")

    if proc_dict['gui']:
        print(f"    - Grafik Arayüz (GUI + alt süreçler, {len(proc_dict['gui'])} süreç): PIDs {[p.pid for p in proc_dict['gui']]}")
    else:
        print("    - [!] Grafik arayüz (GUI) çalışmıyor olarak görünüyor.")

    # Prime cpu_percent
    for p in all_procs:
        try:
            p.cpu_percent(interval=None)
        except Exception:
            pass

    print("\n" + "-" * 70)
    print(">>> TEST BAŞLADI!")
    print(">>> LÜTFEN ŞİMDİ UYGULAMA PENCERESİNDE SEKME VE SAYFALAR ARASINDA GEZİNİN...")
    print(">>> (Dashboard, Performans Modları, Klavye Işıkları, Ekran, Pil vb.)")
    print("-" * 70 + "\n")

    samples_cpu = []
    samples_ram = []
    samples_gui_ram = []
    samples_daemon_ram = []

    start_time = time.time()
    sample_count = 0
    total_samples = int(duration / interval)

    try:
        while True:
            elapsed = time.time() - start_time
            if elapsed >= duration:
                break

            time.sleep(interval)
            sample_count += 1

            # Refresh child processes dynamically in case renderer/threads spawned
            current_dict = find_target_processes(target)
            d_cpu, d_ram = sample_resources(current_dict['daemon'])
            g_cpu, g_ram = sample_resources(current_dict['gui'])

            total_cpu = d_cpu + g_cpu
            total_ram = d_ram + g_ram

            samples_cpu.append(total_cpu)
            samples_ram.append(total_ram)
            samples_gui_ram.append(g_ram)
            samples_daemon_ram.append(d_ram)

            avg_cpu = sum(samples_cpu) / len(samples_cpu)
            peak_cpu = max(samples_cpu)
            avg_ram = sum(samples_ram) / len(samples_ram)
            peak_ram = max(samples_ram)

            remaining = max(0.0, duration - elapsed)
            bar_len = 20
            progress = min(1.0, elapsed / duration)
            filled = int(bar_len * progress)
            bar = '#' * filled + '-' * (bar_len - filled)

            sys.stdout.write(
                f"\r[{bar}] {remaining:4.1f}s | "
                f"CPU: %{total_cpu:4.1f} (Ort: %{avg_cpu:4.1f}, Zirve: %{peak_cpu:4.1f}) | "
                f"RAM: {total_ram:5.1f} MB (Ort: {avg_ram:5.1f} MB, Zirve: {peak_ram:5.1f} MB) "
            )
            sys.stdout.flush()

    except KeyboardInterrupt:
        print("\n\n[!] Test kullanıcı tarafından erken sonlandırıldı.")

    print("\n\n" + "=" * 70)
    print("  TEST TAMAMLANDI - SONUÇ ÖZETİ")
    print("=" * 70)

    if not samples_ram:
        print("Yetersiz örneklem toplandı.")
        return {}

    stats = {
        'target': target,
        'app_name': app_name,
        'duration_sec': round(elapsed, 1),
        'sample_count': len(samples_ram),
        'cpu_avg': round(sum(samples_cpu) / len(samples_cpu), 2),
        'cpu_peak': round(max(samples_cpu), 2),
        'cpu_min': round(min(samples_cpu), 2),
        'ram_start': round(samples_ram[0], 2),
        'ram_avg': round(sum(samples_ram) / len(samples_ram), 2),
        'ram_peak': round(max(samples_ram), 2),
        'ram_min': round(min(samples_ram), 2),
        'ram_growth': round(max(samples_ram) - samples_ram[0], 2),
        'gui_ram_avg': round(sum(samples_gui_ram) / len(samples_gui_ram), 2) if samples_gui_ram else 0,
        'daemon_ram_avg': round(sum(samples_daemon_ram) / len(samples_daemon_ram), 2) if samples_daemon_ram else 0,
    }

    print(f"Uygulama            : {stats['app_name']}")
    print(f"Ölçüm Süresi        : {stats['duration_sec']} saniye ({stats['sample_count']} örneklem)")
    print("-" * 70)
    print(f"Ortalama CPU Kullanımı : %{stats['cpu_avg']:.2f}")
    print(f"Zirve (Peak) CPU       : %{stats['cpu_peak']:.2f}")
    print(f"Ortalama Toplam RAM    : {stats['ram_avg']:.2f} MB")
    print(f"  - GUI Ort. RAM       : {stats['gui_ram_avg']:.2f} MB")
    print(f"  - Daemon Ort. RAM    : {stats['daemon_ram_avg']:.2f} MB")
    print(f"Zirve (Peak) RAM       : {stats['ram_peak']:.2f} MB")
    print(f"Gezinme RAM Artışı     : {stats['ram_growth']:+.2f} MB")
    print("=" * 70)

    # Save to JSON
    save_result(target, stats)
    return stats

def save_result(target: str, data: Dict[str, Any]):
    results = {}
    if os.path.exists(RESULTS_FILE):
        try:
            with open(RESULTS_FILE, 'r') as f:
                results = json.load(f)
        except Exception:
            results = {}

    results[target] = data
    with open(RESULTS_FILE, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\n[+] Sonuçlar '{RESULTS_FILE}' dosyasına kaydedildi.")

def show_comparison():
    if not os.path.exists(RESULTS_FILE):
        print(f"\n[!] Henüz kaydedilmiş benchmark sonucu bulunamadı ({RESULTS_FILE}).")
        print("    Lütfen önce hem 'tuxedo' hem de 'monster' testlerini çalıştırın.")
        return

    with open(RESULTS_FILE, 'r') as f:
        data = json.load(f)

    if 'tuxedo' not in data or 'monster' not in data:
        print("\n[!] Karşılaştırma için her iki testin de tamamlanması gerekir:")
        print(f"    - Tuxedo Testi : {'TAMAMLANDI' if 'tuxedo' in data else 'EKSİK'}")
        print(f"    - Monster Testi: {'TAMAMLANDI' if 'monster' in data else 'EKSİK'}")
        print("\nÇalıştırmak için: python3 benchmark.py [tuxedo|monster] [süre]")
        return

    t = data['tuxedo']
    m = data['monster']

    print("\n" + "=" * 80)
    print("                  BENCHMARK KARŞILAŞTIRMA RAPORU")
    print("=" * 80)
    print(f"{'Metrik':<30} | {'Tuxedo CC (Electron)':<22} | {'Monster CC (Tauri/Rust)':<22}")
    print("-" * 80)
    print(f"{'Ortalama Toplam RAM (RSS)':<30} | {t['ram_avg']:>18.1f} MB | {m['ram_avg']:>18.1f} MB")
    print(f"{'Zirve (Peak) RAM':<30} | {t['ram_peak']:>18.1f} MB | {m['ram_peak']:>18.1f} MB")
    print(f"{'  * GUI Ortalama RAM':<30} | {t['gui_ram_avg']:>18.1f} MB | {m['gui_ram_avg']:>18.1f} MB")
    print(f"{'  * Daemon Ortalama RAM':<30} | {t['daemon_ram_avg']:>18.1f} MB | {m['daemon_ram_avg']:>18.1f} MB")
    print(f"{'Ortalama CPU Kullanımı':<30} | {t['cpu_avg']:>17.2f} % | {m['cpu_avg']:>17.2f} %")
    print(f"{'Zirve (Peak) CPU':<30} | {t['cpu_peak']:>17.2f} % | {m['cpu_peak']:>17.2f} %")
    print(f"{'Sayfa Gezinmesi RAM Şişmesi':<30} | {t['ram_growth']:>17.1f} MB | {m['ram_growth']:>17.1f} MB")
    print("=" * 80)

    # Calculate differences
    if t['ram_avg'] > 0:
        ram_saving = ((t['ram_avg'] - m['ram_avg']) / t['ram_avg']) * 100
        print(f"\n[*] Bellek Tasarrufu : Monster CC, Tuxedo CC'ye göre %{ram_saving:.1f} daha az RAM kullanıyor.")
    if t['daemon_ram_avg'] > 0:
        daemon_saving = ((t['daemon_ram_avg'] - m['daemon_ram_avg']) / t['daemon_ram_avg']) * 100
        print(f"[*] Daemon Verimliliği: Monster arka plan servisi %{daemon_saving:.1f} daha hafif.")
    if t['cpu_avg'] > 0:
        cpu_diff = ((t['cpu_avg'] - m['cpu_avg']) / t['cpu_avg']) * 100
        print(f"[*] CPU Tüketim Farkı: Monster CC sayfa gezinmesinde %{cpu_diff:.1f} daha az CPU harcıyor.")
    print("=" * 80 + "\n")

def main():
    parser = argparse.ArgumentParser(description="Tuxedo vs Monster Control Center Benchmark")
    parser.add_argument("target", nargs="?", choices=["tuxedo", "monster", "compare"], help="Test edilecek hedef veya karşılaştırma")
    parser.add_argument("duration", nargs="?", type=int, default=30, help="Test süresi (saniye, varsayılan 30)")
    args = parser.parse_args()

    if args.target == "compare":
        show_comparison()
        return

    if args.target in ["tuxedo", "monster"]:
        run_benchmark(args.target, duration=args.duration)
        return

    # Interactive menu
    print("\n" + "=" * 60)
    print("      KONTROL MERKEZİ BENCHMARK VE KARŞILAŞTIRMA")
    print("=" * 60)
    print("  1) Tuxedo Control Center Testi Yap (Örn: 30 saniye)")
    print("  2) Monster Control Center Testi Yap (Örn: 30 saniye)")
    print("  3) Karşılaştırma Raporunu Görüntüle")
    print("  4) Çıkış")
    print("-" * 60)

    choice = input("Seçiminiz [1-4]: ").strip()
    if choice == "1":
        dur = input("Test süresi saniye cinsinden [varsayılan 30]: ").strip()
        dur = int(dur) if dur.isdigit() else 30
        run_benchmark("tuxedo", duration=dur)
    elif choice == "2":
        dur = input("Test süresi saniye cinsinden [varsayılan 30]: ").strip()
        dur = int(dur) if dur.isdigit() else 30
        run_benchmark("monster", duration=dur)
    elif choice == "3":
        show_comparison()
    else:
        print("Çıkılıyor.")

if __name__ == "__main__":
    main()
