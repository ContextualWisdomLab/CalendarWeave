"""Real finite child-only loopback probe; never loads database credentials."""
import json
import os
from pathlib import Path
import selectors
import socket
import subprocess
import sys
import time


OWNERS = []


class Owner:
    """Strong owner of one cooperative constructor-returned immediate child."""

    def __init__(self, child, cutoff):
        self.child, self.cutoff = child, cutoff
        self.waited = False
        self.errors = []
        self.primary = None
        self.resources = []

    def remaining(self, cap):
        return max(0, min(cap, self.cutoff - time.monotonic()))

    def wait(self):
        value = self.child.wait(timeout=self.remaining(5))
        if type(value) is not int or type(self.child.returncode) is not int or value != self.child.returncode:
            raise RuntimeError('exact child wait not established')
        self.waited = True
        return value

    def cleanup(self, result):
        if not self.waited:
            for signal in (self.child.terminate, self.child.kill):
                try:
                    signal()
                except BaseException as error:
                    self.errors.append(error)
                try:
                    result['cleanup_wait'] = self.wait()
                except BaseException as error:
                    self.errors.append(error)
                if self.waited:
                    break
        result['returncode'] = self.child.returncode
        for label, resource in self.resources:
            try:
                if resource is not None:
                    resource.close()
                result[label + '_closed'] = True
            except BaseException as error:
                self.errors.append(error)
                result[label + '_closed'] = False
        result['exact_wait'] = self.waited
        if self.waited:
            OWNERS.remove(self)


def probe(binary, request, drip=False):
    if OWNERS:
        raise RuntimeError('unsettled exact child: new launch refused')
    # Do not copy inherited provider/database credentials into the fixture.
    env = {'PATH': '/usr/bin:/bin', 'LANG': 'C', 'LC_ALL': 'C'}
    # Invalid synthetic configuration must be ignored by serve, not connected.
    env['CALENDARWEAVE_DATABASE_URL'] = 'invalid-synthetic-do-not-connect'
    argv = [str(binary), 'serve', '127.0.0.1:0', '--once']
    result = {'argv': argv}
    owner = Owner(None, time.monotonic() + 15)
    owner.child = subprocess.Popen(argv, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    OWNERS.append(owner)
    child = owner.child
    selector = None
    primary = None
    try:
        owner.resources = [('selector', None), ('stdout', child.stdout), ('stderr', child.stderr)]
        result['pid'] = child.pid
        selector = selectors.DefaultSelector()
        owner.resources[0] = ('selector', selector)
        selector.register(child.stdout, selectors.EVENT_READ)
        deadline = time.monotonic() + 5
        line = b''
        while not line.endswith(b'\n'):
            remaining = deadline - time.monotonic()
            assert remaining > 0, 'listener publication deadline'
            assert selector.select(remaining), 'listener publication deadline'
            byte = os.read(child.stdout.fileno(), 1)
            assert byte, 'serve did not publish a listener (missing executable serve)'
            line += byte
            assert len(line) <= 128, 'bounded publication'
        assert line.startswith(b'LISTENING '), 'actual bound-address publication'
        address = line[len(b'LISTENING '):].strip().decode('ascii')
        host, port = address.rsplit(':', 1)
        assert host == '127.0.0.1' and 0 < int(port) <= 65535
        result['bound_address'] = address
        with socket.create_connection((host, int(port)), timeout=3) as connection:
            connection.settimeout(4)
            connection.sendall(request)
            response = b''
            while True:
                data = connection.recv(4096)
                if not data:
                    break
                response += data
                assert len(response) <= 16384, 'bounded response'
        out, err = child.communicate(timeout=owner.remaining(5))
        result.update(response=response.decode('ascii'), stdout_tail=out.decode('ascii'), stderr=err.decode('ascii'))
        result['wait'] = owner.wait()
        assert result['wait'] == 0, 'serve failed'
        return result
    except BaseException as error:
        primary = error
        raise
    finally:
        owner.primary = primary
        owner.cleanup(result)
        result['settlement_ok'] = owner.waited and not owner.errors
        try:
            print(json.dumps(result), flush=True)
        except BaseException as error:
            owner.errors.append(error)
        if primary is None and owner.errors:
            raise owner.errors[0]


def _main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    binary = Path(argv[0])
    request = b'GET /calendars/cal_example/evt_example HTTP/1.1\r\nHost: localhost\r\n\r\n'
    if len(argv) > 1 and argv[1] == 'idle':
        observed = probe(binary, b'')
        assert observed['response'].startswith('HTTP/1.1 408 Request Timeout\r\n')
        return 0
    observed = probe(binary, request)
    response = observed['response']
    assert response.startswith('HTTP/1.1 503 Service Unavailable\r\n'), 'default unavailable status'
    assert response.endswith('\r\n\r\nAuthorizationUnavailable\n'), 'bounded failure body'
    assert '\r\nETag:' not in response
    assert 'BEGIN:VCALENDAR' not in response
    assert observed['stderr'] == ''
    return 0


def await_settlement():
    """Remain the exact child's living custodian; EOF is not permission to exit."""
    while OWNERS:
        try:
            try:
                print('UNSETTLED: owner alive; status/reconcile; exit refused', file=sys.stderr, flush=True)
            except BaseException:
                pass
            command = sys.stdin.readline()
            if command.strip() == 'reconcile':
                for owner in tuple(OWNERS):
                    try:
                        owner.wait()
                    except BaseException as error:
                        owner.errors.append(error)
                    if owner.waited:
                        OWNERS.remove(owner)
            elif not command:
                time.sleep(1)
        except BaseException as error:
            # Ctrl-C, read errors and EOF never relinquish wait rights.
            for owner in OWNERS:
                owner.errors.append(error)
            time.sleep(1)


def main(argv=None):
    try:
        return _main(argv)
    except BaseException:
        await_settlement()
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
