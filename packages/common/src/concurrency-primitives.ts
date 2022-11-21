import { EventEmitter, on } from 'node:events';
import { IllegalStateError } from './errors';

/**
 * A counting semaphore.
 */
export class CountingSemaphore {
  private readonly emitter = new EventEmitter();
  private readonly controller = new AbortController();
  private readonly releaseEvents = on(this.emitter, 'released', { signal: this.controller.signal })[
    Symbol.asyncIterator
  ]();

  private value: number;

  constructor(initialValue: number) {
    this.value = initialValue;
  }

  public async acquire(): Promise<void> {
    while (this.value <= 0) await this.releaseEvents.next();
    this.value--;
  }

  public tryAcquire(): boolean {
    if (this.value <= 0) return false;
    this.value--;
    return true;
  }

  public release(): void {
    this.value++;
    this.emitter.emit('released');
  }

  public abort(): void {
    this.controller.abort();
  }
}

/**
 * A queue that can be awaited on both read (ie. when it is empty) and write (ie. when it is full).
 */
export class BoundedBlockingQueue<T> {
  private state: 'OPEN' | 'CLOSING' | 'CLOSED' = 'OPEN';
  private readonly queue: T[] = [];
  private isNotEmptyCondition = new WaitableCondition(false);
  private isNotFullCondition = new WaitableCondition(true);

  constructor(public readonly capacity: number) {}

  public async push(val: T): Promise<void> {
    if (this.state !== 'OPEN') throw new IllegalStateError('BoundedBlockingQueue is being closed or has been closed');
    for (;;) {
      // Cast needed because TypeScript is not realizing that value of this.state
      // may change while waiting on the this.isNotFullCondition promise below.
      if ((this.state as string) === 'CLOSED')
        throw new IllegalStateError('BoundedBlockingQueue has been closed while waiting to push a new value');

      if (this.queue.length < this.capacity) {
        this.queue.push(val);
        this.isNotEmptyCondition.value = true;
        return;
      }
      await this.isNotFullCondition;
    }
  }

  public async take(): Promise<T | null> {
    for (;;) {
      if (this.queue.length > 0) {
        return this.queue.shift() as T;
      }
      if (this.state === 'CLOSED' || this.state === 'CLOSING') {
        this.state = 'CLOSED';
        return null;
      }
      await this.isNotEmptyCondition;
    }
  }

  close() {
    this.state = 'CLOSING';
  }
}
