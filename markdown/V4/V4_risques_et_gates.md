# Risques et gates — V4

## 1. Objet

Identifier les risques majeurs de la V4 et définir les gates de passage vers la V5.

---

## 2. Risques majeurs

## 2.1 Dopamine mal modélisée
Risque :
- dopamine trop simple → effet nul ;
- dopamine trop forte → système instable ;
- dopamine détournée en pseudo-contrôle.

Mitigation :
- commencer par un scalaire minimal ;
- garder le canal désactivable ;
- séparer modulation de plasticité et contrôle d’activité.

## 2.2 Eligibility trop diffuse
Risque :
- presque toutes les arêtes gardent une mémoire récente ;
- perte de sélectivité.

Mitigation :
- bornage ;
- decay net ;
- seuils ;
- analyse distributions.

## 2.3 I/O artificielles
Risque :
- patterns trop triviales ;
- readout qui contourne le substrat.

Mitigation :
- protocoles simples mais non dégénérés ;
- groupe de sortie clair ;
- comparaison substrat intact vs readout forcé.

## 2.4 Reward sans impact
Risque :
- le reward existe mais n’influence presque rien.

Mitigation :
- comparaison stricte STDP seule vs STDP+reward ;
- tâches où la différence doit être visible.

## 2.5 Explosion paramétrique
Risque :
- trop de nouveaux gains, decays, seuils.

Mitigation :
- profils minimaux ;
- mécanismes désactivables ;
- campagnes d’ablation.

---

## 3. Gate de sortie V4

Passage à V5 autorisé uniquement si :

1. reward influence réellement les poids ;
2. reward influence réellement le comportement de sortie ;
3. traces d’éligibilité utiles démontrées ;
4. entrée et sortie minimales interprétables ;
5. stabilité globale V3 conservée ;
6. points ouverts post-V3 suffisamment tranchés.

---

## 4. Si le gate échoue

- si reward OK mais I/O faibles : itération V4 centrée I/O ;
- si I/O OK mais reward sans effet : itération V4 centrée dopamine/eligibility ;
- si tout marche sauf coût : sprint GPU transversal avant V5.
