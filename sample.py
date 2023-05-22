import aioitertools
import asyncio
import itertools
from python_async_iterator import fibonacci_sync, fibonacci_async, struct_sync, cat_async

def run_sync():
    print("Sync iterator:")
    iter_sync = fibonacci_sync()

    for n in itertools.islice(iter_sync, 20):
        print(n)

async def run_async():
    print("Async iterator:")
    iter_async = fibonacci_async()

    async for n in aioitertools.islice(iter_async, 20):
        print(n)

def run_struct_sync():
    print("Sync iterator over a struct:")
    iter_sync = struct_sync()

    for n in itertools.islice(iter_sync, 20):
        print(f'{n.msg} - {n.time}')

async def run_cat_async():
    print("Async cat iterator:")
    iter_async = cat_async()

    async for n in aioitertools.islice(iter_async, 50):
        print(n)

run_sync()
run_struct_sync()
asyncio.run(run_async())
asyncio.run(run_cat_async())
