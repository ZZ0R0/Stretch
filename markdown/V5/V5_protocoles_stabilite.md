# Protocoles de stabilité — V5

## 1. Objet

Définir les tests permettant de vérifier que la V5 reste stable malgré :

- calibration adaptative ;
- dynamique soutenue ;
- tâches anti-biais ;
- diagnostics supplémentaires.

---

## 2. Questions de stabilité

1. Les lois adaptatives restent-elles stables quand `n` augmente ?
2. Le maintien d’activité ne provoque-t-il pas une saturation ?
3. Les chemins renforcés restent-ils lisibles sans divergence ?
4. Les baselines restent-elles comparables au modèle complet ?

---

## 3. Campagnes de test

## 3.1 Test A — stabilité multi-échelle

Échelles :
- 50k
- 500k
- 5M

Mesures :
- énergie totale ;
- ratio nœuds actifs ;
- mémoire utilisée ;
- temps par tick ;
- sustain ratio.

Attendu :
- comportement qualitativement cohérent ;
- pas de divergence brutale avec l’échelle.

---

## 3.2 Test B — dynamique soutenue

Comparer :
- V4-like decay ;
- decay adaptatif ;
- réverbération locale ;
- feedback local.

Mesures :
- sustain ratio ;
- durée d’activité après stimulus ;
- saturation ;
- fragmentation spatiale.

Attendu :
- activité moins impulsionnelle ;
- pas d’emballement global.

---

## 3.3 Test C — re-routing

Tâche :
- association inversée.

Mesures :
- accuracy ;
- temps de convergence ;
- cohérence des chemins ;
- évolution des conductances.

Attendu :
- preuve d’apprentissage sans collapse.

---

## 4. Indicateurs d’échec

La V5 est instable si :
- sustain ratio monte mais au prix d’une saturation globale ;
- les lois adaptatives explosent pour 500k ou 5M ;
- les chemins deviennent illisibles ;
- les diagnostics montrent du renforcement diffus non ciblé.
