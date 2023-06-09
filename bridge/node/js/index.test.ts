import { printHelloGraph } from './index';

test('printHelloGraph should print "Hello, Graph!"', () => {
    // Mock the console.log function
    const consoleLogMock = jest.spyOn(console, 'log').mockImplementation();
  
    // Call the printHelloGraph function
    printHelloGraph();

  });
  