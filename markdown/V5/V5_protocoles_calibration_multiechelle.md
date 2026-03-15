# Protocoles calibration multi-échelle — V5

## 1. Objet

Valider que les paramètres ne sont plus figés artificiellement à une seule taille de réseau.

---

## 2. Hypothèse

Des lois adaptatives simples sur :
- gain ;
- eligibility decay ;
- edge decay ;
- target activity ;
- presentation ticks ;
- group size

peuvent améliorer la robustesse entre 50k, 500k et 5M.

---

## 3. Campagne recommandée

Échelles :
- 50k
- 500k
- 5M

Pour chaque échelle :
1. baseline V4 fixe ;
2. profil adaptatif A ;
3. profil adaptatif B ;
4. profil adaptatif C.

---

## 4. Mesures

- accuracy ;
- reward cumulé ;
- sustain ratio ;
- temps par tick ;
- cohérence topologique CT ;
- stabilité globale ;
- proportion d’arêtes modifiées.

---

## 5. Paramètres candidats

- `group_size`
- `propagation_gain`
- `eligibility_decay`
- `edge_decay`
- `spatial_lambda`
- `target_activity`
- `presentation_ticks`

---

## 6. Critères de réussite

- amélioration nette sur au moins deux échelles ;
- réduction de la dégradation entre 50k et 5M ;
- pas d’explosion de coût ou d’instabilité.
