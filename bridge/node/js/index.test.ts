import { Graph } from './graph';
import { Config } from './models/config';
import { DevEnvironment, Environment, EnvironmentType } from './models/environment';

test('printHelloGraph should print "Hello, Graph!"', () => {
    // Mock the console.log function
    const consoleLogMock = jest.spyOn(console, 'log').mockImplementation();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: {} as Config };
    const graph = new Graph(environment);
    graph.printHelloGraph();
    expect(consoleLogMock).toHaveBeenCalledWith('Hello, Graph!');
  });
  