# Protocoles reward & eligibility — V4

## 1. Objet

Valider que la combinaison :

- STDP
- trace d’éligibilité
- reward
- dopamine

permet un apprentissage guidé.

---

## 2. Hypothèse centrale

Si un sous-réseau vient juste d’être actif et qu’un reward positif survient peu après, alors les arêtes récemment éligibles doivent être plus renforcées qu’en absence de reward.

---

## 3. Expériences

## 3.1 STDP seule vs STDP + reward

Procédure :
1. exécuter une tâche simple avec STDP seule ;
2. mesurer les poids appris ;
3. répéter avec reward + eligibility ;
4. comparer.

Mesures :
- Δw moyen ;
- accuracy ;
- récompense cumulée.

Attendu :
- les poids appris diffèrent ;
- la politique de sortie s’améliore.

---

## 3.2 Reward retardé

Procédure :
- pattern utile à t ;
- reward à t + delta.

Faire varier delta.

Mesures :
- impact sur poids ;
- impact sur output.

Attendu :
- fenêtre d’éligibilité observable.

---

## 3.3 Reward positif vs négatif

Procédure :
- mêmes patterns ;
- reward positif sur certaines séquences ;
- reward négatif sur d’autres.

Mesures :
- asymétrie des poids ;
- comportement sortie.

Attendu :
- différenciation nette.

---

## 4. Critères de réussite

- la reward-modulation influence réellement les poids ;
- l’effet dépend du délai ;
- reward positif et négatif n’ont pas le même impact ;
- l’eligibility ne devient pas un simple bruit diffus.
