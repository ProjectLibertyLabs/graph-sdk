# Graph SDK Configuration
<p>
All of different settings and configuration parameters are encapsulated inside this module.
</p>

#### Supported `Environments` and their corresponding configurations:
- `Mainnet`: This environment corresponds to the **production** settings. It is used for live deployments and real-world
usage.
- `Rococo`: This environment represents the **staging** settings. It is typically used for pre-production testing and
validation before deployment to the production environment.
- `Dev`: This environment is specifically designed for **local** development and testing purposes. It allows
developers to experiment and iterate on their code locally without affecting the production or staging environments.

These different environments provide flexibility and allow developers to adapt their application to specific stages of
development, testing, and production, ensuring a smooth transition from development to deployment.

### Graphs
We are following [DSNP specification](https://spec.dsnp.org/DSNP/Graph.html) for social graph implementation. In
**DSNP** each graph type has a defined schema which describes how it should be serialized and deserialized. For more
details please refer to provided specification link.

Supported graph types are as below:
 - `Public Follow` : One sided graph connections which are public
 - `Public Friendship`: Two sided graph connections which are public
 - `Private Follow`: One sided graph connections which are private (stored encrypted)
 - `Private Frienship`: Two sided graph connections which are private (stored encrypted)


### Errors
All the errors used inside `Graph Sdk` is also defined in this module to allow easy access.