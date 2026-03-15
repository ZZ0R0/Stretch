# V6 — Protocoles d'évaluation

## Protocole 1 : Vérification du front d'onde

**Objectif** : Vérifier que le signal traverse le réseau sous contrainte de sparsité.

1. Lancer `config_v6.toml` en mode single-run
2. Observer que `max(readout_scores) > 0.1` dans au moins 80% des trials
3. Vérifier que l'accuracy dépasse 50% (le hasard)

**Commande** :
```bash
cargo run --release --bin stretch-cli -- configs/config_v6.toml
```

## Protocole 2 : Respect de la sparsité

**Objectif** : Vérifier que le budget de sparsité est respecté.

1. Lancer en mode single-run
2. Vérifier les métriques : `active_nodes / num_nodes ≤ max_active_fraction × 1.05`
3. La moyenne sur 100 ticks doit être ≤ fraction × num_nodes

## Protocole 3 : Multi-seed benchmark

**Objectif** : Vérifier la significativité de l'apprentissage.

1. Lancer `config_v6.toml --seeds 5`
2. Vérifier : accuracy moyenne > 55%, écart-type < 15%

**Commande** :
```bash
cargo run --release --bin stretch-cli -- configs/config_v6.toml --seeds 5
```

## Protocole 4 : Non-régression V5

**Objectif** : Vérifier que les tests V5 passent toujours.

```bash
cargo test
```

10/10 tests doivent passer (la V6 est opt-in par config).

## Protocole 5 : Build propre

```bash
cargo build --release 2>&1 | grep -E "error|warning"
cargo clippy 2>&1 | grep -E "error|warning"
```

0 erreurs, 0 warnings.
