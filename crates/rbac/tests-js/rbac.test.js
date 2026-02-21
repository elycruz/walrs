import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { JsRbac, JsRbacBuilder, createRbacFromJson, checkPermission } from '../pkg/walrs_rbac.js';

describe('JsRbacBuilder', () => {
  describe('Constructor and Basic Building', () => {
    test('should create an empty RBAC builder', () => {
      const builder = new JsRbacBuilder();
      assert.ok(builder, 'Builder should be created');

      const rbac = builder.build();
      assert.ok(rbac instanceof JsRbac, 'Should build an RBAC instance');
    });

    test('should add a role with permissions', () => {
      const rbac = new JsRbacBuilder()
        .addRole('admin', ['manage.users'], null)
        .build();

      assert.ok(rbac.hasRole('admin'), 'RBAC should have the admin role');
      assert.ok(rbac.isGranted('admin', 'manage.users'), 'Admin should have manage.users');
    });

    test('should add a role with children', () => {
      const rbac = new JsRbacBuilder()
        .addRole('guest', ['read'], null)
        .addRole('user', ['write'], ['guest'])
        .build();

      assert.ok(rbac.hasRole('guest'), 'RBAC should have guest role');
      assert.ok(rbac.hasRole('user'), 'RBAC should have user role');
      assert.ok(rbac.isGranted('user', 'write'), 'User should have write permission');
      assert.ok(rbac.isGranted('user', 'read'), 'User should inherit read from guest');
      assert.ok(!rbac.isGranted('guest', 'write'), 'Guest should not have write');
    });
  });

  describe('Permission Inheritance', () => {
    test('should support deep inheritance', () => {
      const rbac = new JsRbacBuilder()
        .addRole('guest', ['read.public'], null)
        .addRole('user', ['write.post'], ['guest'])
        .addRole('editor', ['edit.post'], ['user'])
        .addRole('admin', ['admin.panel'], ['editor'])
        .build();

      // Admin inherits all
      assert.ok(rbac.isGranted('admin', 'admin.panel'));
      assert.ok(rbac.isGranted('admin', 'edit.post'));
      assert.ok(rbac.isGranted('admin', 'write.post'));
      assert.ok(rbac.isGranted('admin', 'read.public'));

      // Editor doesn't have admin
      assert.ok(!rbac.isGranted('editor', 'admin.panel'));
      assert.ok(rbac.isGranted('editor', 'edit.post'));
      assert.ok(rbac.isGranted('editor', 'write.post'));

      // Guest only has read
      assert.ok(rbac.isGranted('guest', 'read.public'));
      assert.ok(!rbac.isGranted('guest', 'write.post'));
    });

    test('should support multiple children', () => {
      const rbac = new JsRbacBuilder()
        .addRole('reader', ['read'], null)
        .addRole('writer', ['write'], null)
        .addRole('admin', ['admin'], ['reader', 'writer'])
        .build();

      assert.ok(rbac.isGranted('admin', 'read'));
      assert.ok(rbac.isGranted('admin', 'write'));
      assert.ok(rbac.isGranted('admin', 'admin'));
    });
  });

  describe('addPermission', () => {
    test('should add permission to existing role', () => {
      const rbac = new JsRbacBuilder()
        .addRole('user', ['read'], null)
        .addPermission('user', 'write')
        .build();

      assert.ok(rbac.isGranted('user', 'read'));
      assert.ok(rbac.isGranted('user', 'write'));
    });
  });

  describe('addChild', () => {
    test('should add child relationship', () => {
      const rbac = new JsRbacBuilder()
        .addRole('editor', ['edit'], null)
        .addRole('admin', ['admin'], null)
        .addChild('admin', 'editor')
        .build();

      assert.ok(rbac.isGranted('admin', 'edit'));
      assert.ok(rbac.isGranted('admin', 'admin'));
    });
  });

  describe('fromJson', () => {
    test('should create builder from JSON', () => {
      const json = JSON.stringify({
        roles: [
          ['guest', ['read'], null],
          ['admin', ['manage'], ['guest']]
        ]
      });

      const builder = JsRbacBuilder.fromJson(json);
      const rbac = builder.build();

      assert.ok(rbac.isGranted('admin', 'manage'));
      assert.ok(rbac.isGranted('admin', 'read'));
    });
  });

  describe('Error Handling', () => {
    test('should throw on invalid JSON', () => {
      assert.throws(() => {
        JsRbacBuilder.fromJson('invalid json');
      });
    });

    test('should throw on missing child role', () => {
      assert.throws(() => {
        new JsRbacBuilder()
          .addRole('admin', ['manage'], ['nonexistent'])
          .build();
      });
    });

    test('should throw on cycle', () => {
      assert.throws(() => {
        new JsRbacBuilder()
          .addRole('a', [], ['b'])
          .addRole('b', [], ['a'])
          .build();
      });
    });
  });
});

describe('JsRbac', () => {
  describe('Constructor', () => {
    test('should create empty RBAC', () => {
      const rbac = new JsRbac();
      assert.equal(rbac.roleCount(), 0);
    });
  });

  describe('fromJson', () => {
    test('should create RBAC from JSON', () => {
      const json = JSON.stringify({
        roles: [
          ['guest', ['read'], null],
          ['admin', ['manage'], ['guest']]
        ]
      });

      const rbac = JsRbac.fromJson(json);

      assert.ok(rbac.hasRole('guest'));
      assert.ok(rbac.hasRole('admin'));
      assert.ok(rbac.isGranted('admin', 'manage'));
      assert.ok(rbac.isGranted('admin', 'read'));
    });

    test('should throw on invalid JSON', () => {
      assert.throws(() => {
        JsRbac.fromJson('not json');
      });
    });
  });

  describe('isGranted', () => {
    test('should return false for non-existent role', () => {
      const rbac = new JsRbac();
      assert.equal(rbac.isGranted('nonexistent', 'anything'), false);
    });
  });

  describe('hasRole', () => {
    test('should return false for non-existent role', () => {
      const rbac = new JsRbac();
      assert.equal(rbac.hasRole('nonexistent'), false);
    });
  });
});

describe('Convenience Functions', () => {
  test('createRbacFromJson should create RBAC', () => {
    const json = JSON.stringify({
      roles: [
        ['admin', ['manage'], null]
      ]
    });

    const rbac = createRbacFromJson(json);
    assert.ok(rbac.isGranted('admin', 'manage'));
  });

  test('checkPermission should check permission directly', () => {
    const json = JSON.stringify({
      roles: [
        ['admin', ['manage'], null]
      ]
    });

    assert.equal(checkPermission(json, 'admin', 'manage'), true);
    assert.equal(checkPermission(json, 'admin', 'other'), false);
  });
});
