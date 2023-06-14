import { Graph } from './graph';
import { Config } from './models/config';
import { DevEnvironment, EnvironmentInterface, EnvironmentType } from './models/environment';


test('printHelloGraph should print "Hello, Graph!"', async () => {
    // Mock the console.log function
    const consoleLogMock = jest.spyOn(console, 'log').mockImplementation();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: {} as Config };
    const graph = new Graph(environment);
    await graph.printHelloGraph();
    expect(consoleLogMock).toHaveBeenCalledWith('Hello, Graph!');
});

test('getGraphConfig should return the graph config', async () => {
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: {} as Config };
    const graph = new Graph(environment);
    const config = await graph.getGraphConfig(environment);
    expect(config).toBeDefined();
    expect(config.sdkMaxUsersGraphSize).toEqual(1000);
});

test('getGraphConfig with Mainnet environment should return the graph config', async () => {
    const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };
    const graph = new Graph(environment);
    const config = await graph.getGraphConfig(environment);
    expect(config).toBeDefined();
    expect(config.sdkMaxUsersGraphSize).toEqual(1000);
});

test('getGraphConfig with Rococo environment should return the graph config', async () => {
  const environment: EnvironmentInterface = { environmentType: EnvironmentType.Rococo };
  const graph = new Graph(environment);
  const config = await graph.getGraphConfig(environment);
  expect(config).toBeDefined();
  expect(config.sdkMaxUsersGraphSize).toEqual(1000);
});

test('initialize graph with low capacity of 100 should return the same capacity', async () => {
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: {} as Config };
    const graph = new Graph(environment, 100);
    const handle = graph.getGraphHandle();
    const capacity = await graph.getGraphCapacity();
    expect(capacity).toEqual(100);
});