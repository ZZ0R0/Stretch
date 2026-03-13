# Protocoles de stabilité — V4

## 1. Objet

Définir les tests permettant de vérifier que la V4 reste stable malgré l’ajout de :
- dopamine ;
- reward ;
- traces d’éligibilité ;
- I/O minimales.

---

## 2. Questions de stabilité

1. La dopamine perturbe-t-elle la stabilité globale ?
2. Le reward fait-il diverger les poids ?
3. Les traces d’éligibilité restent-elles bornées ?
4. L’ajout d’I/O dégrade-t-il l’auto-entretien de V3 ?
5. La consolidation guidée devient-elle plus sélective ou plus instable ?

---

## 3. Campagnes de test

## 3.1 Test A — dopamine seule

Activer :
- dopamine tonique ;
- dopamine phasique sans reward d’apprentissage utile.

Mesures :
- énergie totale ;
- variance d’activité ;
- fraction de nœuds actifs ;
- distribution des poids ;
- distribution des `e_ij`.

Attendu :
- modulation faible à modérée ;
- pas de divergence globale.

---

## 3.2 Test B — reward + eligibility

Faire varier :
- `eta_rew`
- `gamma_e`
- `lambda_d`
- `d_thresh`

Mesures :
- bornes des poids ;
- proportion d’arêtes consolidées ;
- reward cumulé ;
- stabilité activité.

Attendu :
- zone stable avec apprentissage guidé.

---

## 3.3 Test C — I/O

Activer :
- zones d’entrée ;
- zones de sortie ;
- tâche binaire simple.

Mesures :
- activité des zones d’entrée ;
- activité globale hors entrée ;
- comportement des zones de sortie ;
- stabilité du fond d’activité.

Attendu :
- l’I/O n’effondre pas la dynamique du substrat.

---

## 3.4 Test D — reward négatif

Injecter des rewards négatifs fréquents.

Mesures :
- distribution `w_ij` ;
- distribution `e_ij` ;
- activité globale ;
- comportement sortie.

Attendu :
- LTD renforcée ou dé-consolidation ciblée, sans collapse complet.

---

## 4. Indicateurs d’échec

Le système V4 est instable si :
- `d(t)` devient un second canal de contrôle d’activation ;
- plus de 80% des poids tendent rapidement vers `w_max` ou `w_min` ;
- la reward-modulation tue l’auto-entretien global ;
- les traces d’éligibilité divergent ;
- les zones d’entrée ou sortie saturent durablement le réseau.
