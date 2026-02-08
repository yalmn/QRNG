#include <math.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct {
  long total_bits;
  long total_events;
  double m1;
  double sigma;
  double B;
  double M;
} AnalysisResults;

void analyze_bitstream(const char *filename) {
  FILE *file = fopen(filename, "r");
  if (!file) {
    perror("Fehler beim Oeffnen der Datei");
    return;
  }

  long *gaps = NULL;
  long gap_count = 0;
  long current_pos = 0;
  long last_event_pos = -1;
  long bit_count = 0;
  long event_count = 0;
  int ch;

  while ((ch = fgetc(file)) != EOF) {
    if (ch == '0' || ch == '1') {
      bit_count++;
      if (ch == '1') {
        event_count++;
        if (last_event_pos != -1) {
          long tau = current_pos - last_event_pos;
          gaps = realloc(gaps, (gap_count + 1) * sizeof(long));
          gaps[gap_count++] = tau;
        }
        last_event_pos = current_pos;
      }
      current_pos++;
    }
  }
  fclose(file);

  if (gap_count < 2) {
    printf("Fehler: Zu wenige Events fuer eine statistische Auswertung.\n");
    free(gaps);
    return;
  }

  double sum_tau = 0;
  double sum_tau_sq = 0;
  for (long i = 0; i < gap_count; i++) {
    sum_tau += gaps[i];
    sum_tau_sq += (double)gaps[i] * gaps[i];
  }

  double m1 = sum_tau / gap_count;
  double m2 = sum_tau_sq / gap_count;
  double variance = m2 - (m1 * m1);
  double sigma = sqrt(variance);

  double B = (sigma - m1) / (sigma + m1);

  double memory_sum = 0;
  for (long i = 0; i < gap_count - 1; i++) {
    memory_sum += ((gaps[i] - m1) * (gaps[i + 1] - m1));
  }
  double M = (1.0 / (gap_count - 1)) * (memory_sum / variance);

  // printf("--- Analyse-Ergebnisse ---\n");
  // printf("Gelesene Bits:       %ld\n", bit_count);
  // printf("Anzahl Events ('1'): %ld\n", event_count);
  // printf("Anzahl Luecken:      %ld\n", gap_count);
  // printf("Mittelwert (m1):     %.4f\n", m1);
  // printf("Standardabweichung:  %.4f\n", sigma);
  printf("Burstiness (B):      %.4f\n", B);
  printf("Memory (M):          %.4f\n", M);

  if (B > 0) {
    printf("Kernaussage: Starke Buendelung der Events festgestellt.\n");
  } else if (B < 0) {
    printf(
        "Kernaussage: Events sind eher gleichmaessig verteilt (periodisch).\n");
  }

  free(gaps);
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    printf("Anwendung: %s <dateiname.bin>\n", argv[0]);
    return 1;
  }

  analyze_bitstream(argv[1]);
  return 0;
}
