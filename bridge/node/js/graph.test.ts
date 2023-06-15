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
    await graph.freeGraphState();
});

test('getGraphConfig should return the graph config', async () => {
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: {} as Config };
    const graph = new Graph(environment);
    const config = await graph.getGraphConfig(environment);
    expect(config).toBeDefined();
    expect(config.sdkMaxUsersGraphSize).toEqual(1000);
    await graph.freeGraphState();
});

test('getGraphConfig with Mainnet environment should return the graph config', async () => {
    const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };
    const graph = new Graph(environment);
    const config = await graph.getGraphConfig(environment);
    expect(config).toBeDefined();
    expect(config.sdkMaxUsersGraphSize).toEqual(1000);
    await graph.freeGraphState();
});

test('getGraphConfig with Rococo environment should return the graph config', async () => {
    const environment: EnvironmentInterface = { environmentType: EnvironmentType.Rococo };
    const graph = new Graph(environment);
    const config = await graph.getGraphConfig(environment);
    expect(config).toBeDefined();
    expect(config.sdkMaxUsersGraphSize).toEqual(1000);
    await graph.freeGraphState();
});

test('initialize graph with low capacity of 100 should return the same capacity', async () => {
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: {} as Config };
    const graph = new Graph(environment, 100);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const capacity = await graph.getGraphCapacity();
    expect(capacity).toEqual(100);
    await graph.freeGraphState();
});

test('getGraphStatesCount should be zero after previous graph is freed', async () => {
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: {} as Config };
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    await graph.freeGraphState();
    const count = await graph.getGraphStatesCount();
    expect(count).toEqual(0);
});

test('getGraphStatesCount should be one after graph is initialized', async () => {
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: {} as Config };
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const count = await graph.getGraphStatesCount();
    expect(count).toEqual(1);
    await graph.freeGraphState();
});

test('getGraphUsersCount should be zero on initialized graph', async () => {
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: {} as Config };
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const count = await graph.getGraphUsersCount();
    expect(count).toEqual(0);
    await graph.freeGraphState();
    await expect(async () => {
      await graph.getGraphUsersCount();
    }).rejects.toThrow('Graph state not found');
});

test('containsUserGraph should return false on initialized graph', async () => {
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: {} as Config };
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const contains = await graph.containsUserGraph(1);
    expect(contains).toEqual(false);
    await graph.freeGraphState();
});

test('removeUserGraph should pass through on initialized graph', async () => {
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: {} as Config };
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    await graph.removeUserGraph(1);
    await graph.freeGraphState();
});


  