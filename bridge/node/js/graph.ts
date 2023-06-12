
import { graphsdkModule } from "./index";
import {Config, SchemaConfig, DsnpVersion} from "./models/config";
import { EnvironmentInterface } from "./models/environment";

export class Graph {
    /// The handle to the native graph state
    private handle: number;
    
    constructor( environment: EnvironmentInterface , capacity?: number ) {
        if ( capacity ) {
            this.handle = graphsdkModule.initializeGraphStateWithCapacity( environment, capacity );
        } else {
            this.handle = graphsdkModule.initializeGraphState( environment );
        }
    }

    public printHelloGraph(): void {
        console.log( graphsdkModule.printHelloGraph() );
    }
}
