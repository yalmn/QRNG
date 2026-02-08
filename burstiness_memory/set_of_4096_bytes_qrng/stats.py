import os
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
import glob

def load_bits_from_file(filepath):
    """Liest Bits robust ein (erkennt ASCII oder Binär)."""
    with open(filepath, 'rb') as f:
        raw_data = f.read()
    
    if len(raw_data) > 0 and sum(1 for b in raw_data[:100] if b in [48, 49]) > 80:
        return [1 if b == 49 else 0 for b in raw_data if b in [48, 49]]
    
    bits = []
    for byte in raw_data:
        for i in range(7, -1, -1): 
            bits.append((byte >> i) & 1)
    return bits

def calculate_goh_barabasi_metrics(bits):
    """Berechnet B und M exakt nach Forschungsnotizen."""
    pos = [i for i, b in enumerate(bits) if b == 1]
    if len(pos) < 5: return None
    
    gaps = np.diff(pos) 
    n = len(gaps)
    
    m1 = np.mean(gaps)                       
    m2 = np.mean(np.square(gaps))            
    variance = m2 - (m1**2)                  
    sigma = np.sqrt(variance)                

    B = (sigma - m1) / (sigma + m1)

    if n < 2 or variance == 0:
        M = 0
    else:
        tau_i = gaps[:-1]
        tau_next = gaps[1:]
        m_sum = np.sum((tau_i - m1) * (tau_next - m1))
        M = (1.0 / (n - 1)) * (m_sum / variance)
        
    return B, M

def create_cloud_visual(B_list, M_list):
    sns.set_theme(style="white")
    fig, ax = plt.subplots(figsize=(12, 9), dpi=300)

    B_jitter = np.array(B_list) + np.random.normal(0, 0.008, len(B_list))
    M_jitter = np.array(M_list) + np.random.normal(0, 0.008, len(M_list))

    sns.kdeplot(x=B_jitter, y=M_jitter, fill=True, cmap="Blues", alpha=0.3, ax=ax, bw_adjust=1.5)

    ax.scatter(B_jitter, M_jitter, c='royalblue', s=25, alpha=0.5, edgecolors='none', label='QRNG Samples (N=100)')

    mean_B, mean_M = np.mean(B_list), np.mean(M_list)
    ax.scatter(mean_B, mean_M, c='darkred', marker='o', s=200, edgecolors='white', linewidth=2, 
               label=f'Zentroid (Ø B:{mean_B:.5f}, Ø M:{mean_M:.5f})', zorder=10)

    ax.scatter(0, 0, c='black', marker='+', s=150, label='Perfekter Zufall (0,0)', zorder=11)

    ax.set_xlim(-1, 1); ax.set_ylim(-1, 1)
    ax.axhline(0, color='black', lw=0.8); ax.axvline(0, color='black', lw=0.8)
    
    plt.title('Statistische Validierung der QRNG-Serie\n(Phasenraum-Analyse nach Goh & Barabási)', fontsize=16, fontweight='bold', pad=25)
    ax.set_xlabel('Burstiness (B)', fontsize=12, labelpad=15)
    ax.set_ylabel('Memory (M)', fontsize=12, labelpad=15)

    props = dict(boxstyle='round', facecolor='white', alpha=0.8, edgecolor='0.9')
    ax.text(-0.95, -0.9, "PERIODISCH / REGELMÄSSIG\n(Typisch für p=0.5)", color='darkblue', fontsize=9, bbox=props)
    ax.text(0.95, 0.9, "BURSTINESS / CLUSTERING\n(Bündelung der Bits)", color='darkred', ha='right', fontsize=9, bbox=props)

    ax.legend(loc='upper right', frameon=True, shadow=True)
    plt.tight_layout()
    plt.savefig('qrng_cloud_analysis_fixed.png')
    print(f"\nDurchschnitt B: {mean_B:.5f}")
    print(f"Durchschnitt M: {mean_M:.5f}")

if __name__ == "__main__":
    files = sorted(glob.glob("qrng_*"))
    B_results, M_results = [], []
    
    print(f"Analysiere {len(files)} Dateien...")
    for f in files:
        if os.path.isfile(f) and not f.endswith('.py'):
            bits = load_bits_from_file(f)
            res = calculate_goh_barabasi_metrics(bits)
            if res:
                B_results.append(res[0]); M_results.append(res[1])
    
    if B_results:
        create_cloud_visual(B_results, M_results)
