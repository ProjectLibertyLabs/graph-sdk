# Graph SDK Configuration
<p>
All of different settings and configuration parameters are encapsulated inside this module.
</p>

#### Supported `Environments` and their related configuration
  - `Mainnet` : which is the **production** related settings
  - `Rococo` : which is the **staging** related settings
  - `Dev` : which is for **local** development and testing

### Graphs and Schemas
Due to `DSNP` implementation over `Frequency` Chain, we are associating each a specific and well-defined Avro schema
with each graph type. All of these schemas are registered on desired Environments and assigned a unique schema id after
registration.

Supported graph types are as below:
 - `Public Follow` : One sided graph connections which are public
 - `Public Friendship`: Two sided graph connections which are public
 - `Private Follow`: One sided graph connections which are private (stored encrypted)
 - `Private Frienship`: Two sided graph connections which are private (stored encrypted)


### Errors
All the errors used inside `Graph Sdk` is also defined in this module to allow easy access.