"""Actor-free regressions through the shipped probe and CLI callers.

Only constructor-returned cooperative fake handles are admitted. Native actors
are forbidden; these cases do not reproduce an orphan or prove native waits.
"""
import contextlib
import importlib.util
import io
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('http_wire_probe', Path(__file__).with_name('http_wire_probe.py'))
probe = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(probe)


def deny_actors(event, args):
    if event in ('subprocess.Popen', 'os.fork', 'os.posix_spawn', 'os.kill', 'os.killpg', 'socket.__new__', 'socket.connect'):
        raise AssertionError('native actor forbidden in custody unit tests')


sys.addaudithook(deny_actors)
REQUEST = b'GET /calendars/cal_example/evt_example HTTP/1.1\r\nHost: localhost\r\n\r\n'
RESPONSE = b'HTTP/1.1 503 Service Unavailable\r\n\r\nAuthorizationUnavailable\n'


class Pipe:
    def __init__(self, events, name):
        self.events, self.name, self.closed = events, name, False

    def fileno(self):
        return 90

    def close(self):
        self.events.append(self.name + '.close')
        self.closed = True


class Child:
    def __init__(self, events, fail_wait=False):
        self.events, self.fail_wait = events, fail_wait
        self.pid, self.returncode = 12345, None
        self.stdout, self.stderr = Pipe(events, 'stdout'), Pipe(events, 'stderr')
        self.primary = RuntimeError('synthetic wait uncertainty')

    def communicate(self, timeout):
        self.events.append('communicate')
        if self.fail_wait:
            raise self.primary
        self.returncode = 0
        return b'', b''

    def wait(self, timeout):
        self.events.append(('wait', timeout))
        if self.fail_wait:
            raise self.primary
        self.returncode = 0
        return 0

    def terminate(self):
        self.events.append('terminate')

    def kill(self):
        self.events.append('kill')


class Selector(Pipe):
    def __init__(self, events):
        super().__init__(events, 'selector')

    def register(self, *args):
        pass

    def select(self, timeout):
        return [True]


class Connection:
    def __enter__(self):
        return self

    def __exit__(self, *args):
        pass

    def settimeout(self, timeout):
        pass

    def sendall(self, request):
        pass

    def recv(self, size):
        value, self.response = self.response, b''
        return value


class CustodyTests(unittest.TestCase):
    def setUp(self):
        self.events = []
        self.child = Child(self.events)
        self.selector = Selector(self.events)
        self.connection = Connection()
        self.connection.response = RESPONSE
        publication = iter(b'LISTENING 127.0.0.1:23456\n')
        self.launches = []

        def launch(argv, **kwargs):
            self.launches.append((argv, kwargs))
            return self.child

        for target, replacement in [
            ('subprocess.Popen', launch),
            ('selectors.DefaultSelector', lambda: self.selector),
            ('socket.create_connection', lambda *a, **k: self.connection),
            ('os.read', lambda *a: bytes([next(publication)])),
        ]:
            module, attr = target.split('.')
            self.enterContext(patch.object(getattr(probe, module), attr, replacement))
        self.enterContext(contextlib.redirect_stdout(io.StringIO()))
        self.enterContext(contextlib.redirect_stderr(io.StringIO()))
        self.addCleanup(lambda: getattr(probe, 'OWNERS', []).clear())

    # unittest.enterContext is unavailable on the declared Python 3.9 minimum.
    def enterContext(self, manager):
        value = manager.__enter__()
        self.addCleanup(manager.__exit__, None, None, None)
        return value

    def test_normal_exact_wait_positive(self):
        row = probe.probe(Path('fake-rust-server'), REQUEST)
        self.assertEqual(row['wait'], 0)
        self.assertEqual(row['returncode'], 0)
        self.assertTrue(all(row[name + '_closed'] for name in ('selector', 'stdout', 'stderr')))
        self.assertEqual(len(self.launches), 1)

    def test_persistent_wait_retains_exact_owner_and_fences_launch(self):
        self.child.fail_wait = True
        caught = None
        try:
            probe.probe(Path('fake-rust-server'), REQUEST)
        except BaseException as error:
            caught = error
        owners = getattr(probe, 'OWNERS', [])
        self.assertTrue(any(owner.child is self.child for owner in owners), 'exact returned handle needs strong custody')
        self.assertIs(caught, self.child.primary, 'body primary must survive cleanup uncertainty')
        self.assertIn('terminate', self.events)
        self.assertIn('kill', self.events)
        self.assertEqual(self.events.count('stdout.close'), 1)
        self.assertEqual(self.events.count('stderr.close'), 1)
        with self.assertRaises(RuntimeError):
            probe.probe(Path('second-forbidden-server'), REQUEST)
        self.assertEqual(len(self.launches), 1)

    def test_main_refuses_exit_and_eof_until_exact_wait_reconciliation(self):
        self.child.fail_wait = True
        observations = []
        commands = iter(['status\n', 'exit\n', '', 'reconcile\n'])
        def read_command():
            observations.append(any(owner.child is self.child for owner in getattr(probe, 'OWNERS', [])))
            return next(commands)
        def idle(seconds):
            observations.append(any(owner.child is self.child for owner in probe.OWNERS))
            self.child.fail_wait = False
        with patch.object(probe.sys, 'stdin', type('Input', (), {'readline': staticmethod(read_command)})()), patch.object(probe.time, 'sleep', idle):
            caught = None
            code = None
            try:
                code = probe.main(['fake-rust-server', 'get'])
            except BaseException as error:
                caught = error
        self.assertEqual(observations, [True, True, True, True, True], 'actual main must stay alive owning exact child across exit/EOF')
        self.assertEqual(code, 1)
        self.assertIsNone(caught)
        self.assertFalse(probe.OWNERS)
        self.assertEqual(self.events.count('stdout.close'), 1)
        self.assertEqual(self.events.count('stderr.close'), 1)

    def test_normal_main_positive_and_minimal_child_environment(self):
        self.assertEqual(probe.main(['fake-rust-server', 'get']), 0)
        self.assertEqual(set(self.launches[0][1]['env']), {'PATH', 'LANG', 'LC_ALL', 'CALENDARWEAVE_DATABASE_URL'})

    def test_cleanup_signal_errors_do_not_skip_wait_or_independent_closes(self):
        self.child.fail_wait = True
        def refuse_signal():
            self.events.append('signal.refused')
            raise PermissionError('synthetic signal refusal')
        self.child.terminate = refuse_signal
        self.child.kill = refuse_signal
        with self.assertRaises(RuntimeError) as caught:
            probe.probe(Path('fake-rust-server'), REQUEST)
        self.assertIs(caught.exception, self.child.primary)
        self.assertEqual(self.events.count('signal.refused'), 2)
        waits = [event[1] for event in self.events if isinstance(event, tuple)]
        self.assertEqual(len(waits), 2)
        self.assertLessEqual(waits[1], waits[0])
        self.assertEqual(self.events.count('selector.close'), 1)
        self.assertEqual(self.events.count('stdout.close'), 1)
        self.assertEqual(self.events.count('stderr.close'), 1)

    def test_cleanup_keeps_body_primary_and_all_secondary_close_objects(self):
        self.child.fail_wait = True
        secondary = OSError('synthetic after-effect close refusal')
        original = self.child.stdout.close
        def fail_after_close():
            original()
            raise secondary
        self.child.stdout.close = fail_after_close
        caught = None
        try:
            probe.probe(Path('fake-rust-server'), REQUEST)
        except BaseException as error:
            caught = error
        self.assertIs(caught, self.child.primary)
        owner = probe.OWNERS[0]
        self.assertIs(owner.primary, self.child.primary)
        self.assertTrue(any(error is secondary for error in owner.errors))
        self.assertTrue(self.child.stdout.closed)
        self.assertEqual(self.events.count('stdout.close'), 1)
        self.assertEqual(self.events.count('stderr.close'), 1)

    def test_each_close_failure_is_nonpassing_and_siblings_are_attempted(self):
        for label in ('selector', 'stdout', 'stderr'):
            with self.subTest(label=label):
                self.setUp()
                resource = {'selector': self.selector, 'stdout': self.child.stdout, 'stderr': self.child.stderr}[label]
                original = resource.close
                def refuse_close():
                    self.events.append(label + '.close.refused')
                    raise OSError('synthetic close refusal')
                resource.close = refuse_close
                code = probe.main(['fake-rust-server', 'get'])
                self.assertEqual(code, 1, 'shipped CLI must reject close failure')
                self.assertEqual(self.child.returncode, 0)
                self.assertEqual(self.events.count('stdout.close') + self.events.count('stdout.close.refused'), 1)
                self.assertEqual(self.events.count('stderr.close') + self.events.count('stderr.close.refused'), 1)
                self.assertFalse(probe.OWNERS)
                resource.close = original


if __name__ == '__main__':
    unittest.main()
