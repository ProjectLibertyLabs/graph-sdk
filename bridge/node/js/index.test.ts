import { printHelloGraph } from './index';

describe('printHelloGraph', () => {
  it('should print "Hello, Graph!"', () => {
    const consoleSpy = jest.spyOn(console, 'log');
    printHelloGraph();
    expect(consoleSpy).toHaveBeenCalledWith('Hello, Graph!');
    consoleSpy.mockRestore();
  });
});
