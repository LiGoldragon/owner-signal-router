# owner-signal-persona-router — architecture

*Owner-only Signal contract for PersonaRouter channel policy.*

---

## 0 · TL;DR

`owner-signal-persona-router` is the policy signal for
PersonaRouter channel authority. It carries owner-only orders that
grant, extend, revoke, or deny channel authority in the router.

Ordinary router observation traffic stays in `signal-persona-router`.
Router-to-Mind adjudication observation stays in the Mind working
contract until that relation is deliberately moved. Runtime actors,
policy evaluation, socket binding, durable grant tables, and command
lowering live in `persona-router`.

The initial surface is deliberately small:

- `Grant(ChannelGrant)` grants a router channel.
- `Extend(ChannelExtension)` extends an existing router channel.
- `Revoke(ChannelRevocation)` revokes an existing router channel.
- `Deny(AdjudicationDenial)` closes an adjudication request without a
  grant.

## 1 · Contract Surface

| Operation | Projected Sema class | Meaning |
|---|---|---|
| `Grant` | `Mutate` | Apply owner authority by creating or replacing a live channel grant. |
| `Extend` | `Mutate` | Change the duration of a live channel grant. |
| `Revoke` | `Retract` | Remove a live channel grant. |
| `Deny` | `Mutate` | Record an owner decision that an adjudication request will not receive a grant. |

The Sema classes above are daemon-side projections. The wire carries
contract-local operation roots only; there is no public `Mutate` or
`Retract` wrapper.

| Reply | Meaning |
|---|---|
| `ChannelGranted` | The router accepted and recorded a channel grant. |
| `ChannelExtended` | The router accepted and recorded a channel extension. |
| `ChannelRevoked` | The router accepted and recorded a channel revocation. |
| `AdjudicationDenied` | The router accepted and recorded an adjudication denial. |
| `ChannelOrderRejected` | The order was understood but rejected by router policy. |
| `RequestUnimplemented` | The request is in the contract but not implemented by the current runtime. |

## 2 · Policy Types

`ChannelEndpoint` names internal component endpoints and external
connection classes using `signal-persona-auth` vocabulary.

`ChannelMessageKind` names route categories that can be covered by a
grant. Owner-order names such as channel grant, extension, revocation,
and denial are intentionally absent from this enum; those are
operations on this owner contract, not message categories being
routed through ordinary channels.

`ChannelDuration` is the requested lifetime: one-shot, permanent, or
time-bound.

## 3 · Boundaries

This repo owns:

- owner-only channel-policy operation roots and payload records;
- owner-only replies and rejection reasons;
- rkyv and NOTA round-trip shape for the policy signal;
- the contract-local operation-kind witness.

This repo does not own:

- `persona-router` daemon actors;
- router durable grant tables;
- bootstrap policy files;
- ordinary router observation traffic in `signal-persona-router`;
- Mind graph, work graph, or adjudication observation records in
  `signal-persona-mind`;
- CLI argv parsing or socket permissions.

## 4 · Constraints

- The contract exposes owner-only router channel-policy operations,
  not ordinary router observation queries.
- Grant, extension, revocation, and denial are owner operations on
  this contract, not message kinds in the routed-channel vocabulary.
- Every operation root is a contract-local verb in verb form.
- The wire shape contains no public Sema wrapper such as `Mutate` or
  `Retract`.
- Channel identifiers are daemon-minted reply data or references to
  existing channels; callers do not mint new channel identifiers for
  grant creation.
- The contract crate contains no runtime actors, database handles,
  sockets, command execution, or policy evaluation logic.

## 5 · Witness Tests

`tests/round_trip.rs` proves:

- request operations round-trip through `OwnerRouterFrame`;
- replies round-trip through `OwnerRouterFrame`;
- NOTA request heads are contract-local verbs;
- owner-order names are absent from `ChannelMessageKind`;
- the public request exposes a contract-owned `OperationKind`.

## Code Map

```text
src/lib.rs            owner router channel-policy types and signal_channel! declaration
tests/round_trip.rs   frame round trips and contract-local operation witnesses
```

## See Also

- `../signal-persona-router/ARCHITECTURE.md`
- `../persona-router/ARCHITECTURE.md`
- `../signal-persona-mind/ARCHITECTURE.md`
- `../signal-frame/ARCHITECTURE.md`
- `../signal-sema/ARCHITECTURE.md`
- `~/primary/skills/contract-repo.md`
- `~/primary/skills/component-triad.md`

