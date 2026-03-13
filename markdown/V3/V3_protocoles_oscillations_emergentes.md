# Protocoles oscillations émergentes — V3

## 1. Objet

Déterminer si la V3 produit des oscillations issues de circuits E/I récurrents sans dépendre des pacemakers.

---

## 2. Hypothèse

Avec :
- neurones excitateurs ;
- neurones inhibiteurs ;
- constantes de temps différenciées ;
- propagation récurrente ;
- PID indirect ;

des oscillations locales peuvent émerger.

---

## 3. Protocole

## 3.1 Sans pacemaker

- désactiver tous les pacemakers ;
- activer le PID indirect ;
- injecter un stimulus bref ;
- observer l’activité.

Mesures :
- FFT de l’activité locale ;
- fréquence dominante ;
- durée de maintien ;
- phase relative E vs I.

---

## 3.2 Avec pacemaker d’amorce seulement

- pacemaker actif sur fenêtre courte ;
- arrêt du pacemaker ;
- observer si l’oscillation persiste.

Attendu :
- persistance au moins transitoire après arrêt.

---

## 3.3 Carte spatiale

Mesurer :
- zones oscillantes ;
- amplitude par zone ;
- cohérence inter-zones.

---

## 4. Critères de réussite

Une oscillation émergente V3 est considérée crédible si :
- elle apparaît sans pacemaker permanent ;
- elle possède une fréquence dominante stable ;
- elle implique une alternance E/I visible ;
- elle est reproductible sur plusieurs runs.
