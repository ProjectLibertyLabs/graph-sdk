import { Graph } from './graph';

test('printHelloGraph should print "Hello, Graph!"', () => {
    // Mock the console.log function
    const consoleLogMock = jest.spyOn(console, 'log').mockImplementation();
    let graph = new Graph();
    graph.printHelloGraph();
    expect(consoleLogMock).toHaveBeenCalledWith('Hello, Graph!');
  });
  