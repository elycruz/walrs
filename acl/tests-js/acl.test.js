import { test, describe, before } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { JsAcl, JsAclBuilder, createAclFromJson, checkPermission } from '../pkg/walrs_acl.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load the extensive ACL fixture
let extensiveAclJson;

before(async () => {

  // Load the extensive ACL fixture
  const fixturePath = join(__dirname, '..', 'test-fixtures', 'example-extensive-acl-array.json');
  extensiveAclJson = await readFile(fixturePath, 'utf-8');
});

describe('JsAclBuilder', () => {
  describe('Constructor and Basic Building', () => {
    test('should create an empty ACL builder', () => {
      const builder = new JsAclBuilder();
      assert.ok(builder, 'Builder should be created');

      const acl = builder.build();
      assert.ok(acl instanceof JsAcl, 'Should build an ACL instance');
    });

    test('should add a single role', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .build();

      assert.ok(acl.hasRole('user'), 'ACL should have the user role');
    });

    test('should add a role with parent', () => {
      const acl = new JsAclBuilder()
        .addRole('guest', null)
        .addRole('user', ['guest'])
        .build();

      assert.ok(acl.hasRole('user'), 'ACL should have the user role');
      assert.ok(acl.inheritsRole('user', 'guest'), 'User should inherit from guest');
    });

    test('should add multiple roles at once', () => {
      const acl = new JsAclBuilder()
        .addRoles([
          ['guest', null],
          ['user', ['guest']],
          ['admin', ['user']]
        ])
        .build();

      assert.ok(acl.hasRole('guest'), 'ACL should have guest role');
      assert.ok(acl.hasRole('user'), 'ACL should have user role');
      assert.ok(acl.hasRole('admin'), 'ACL should have admin role');
      assert.ok(acl.inheritsRole('admin', 'user'), 'Admin should inherit from user');
      assert.ok(acl.inheritsRole('user', 'guest'), 'User should inherit from guest');
    });

    test('should add a single resource', () => {
      const acl = new JsAclBuilder()
        .addResource('blog', null)
        .build();

      assert.ok(acl.hasResource('blog'), 'ACL should have the blog resource');
    });

    test('should add a resource with parent', () => {
      const acl = new JsAclBuilder()
        .addResource('public_pages', null)
        .addResource('blog', ['public_pages'])
        .build();

      assert.ok(acl.hasResource('blog'), 'ACL should have the blog resource');
      assert.ok(acl.inheritsResource('blog', 'public_pages'), 'Blog should inherit from public_pages');
    });

    test('should add multiple resources at once', () => {
      const acl = new JsAclBuilder()
        .addRole('guest', null)
        .addResources([
          ['media_library', null],
          ['image', ['media_library']],
          ['video', ['media_library']]
        ])
        .allow(['guest'], ['media_library'], ['read'])
        .build();

      assert.ok(acl.hasResource('media_library'), 'ACL should have media_library resource');
      assert.ok(acl.hasResource('image'), 'ACL should have image resource');
      assert.ok(acl.hasResource('video'), 'ACL should have video resource');
      assert.ok(acl.inheritsResource('image', 'media_library'), 'Image should inherit from media_library');
      assert.ok(acl.inheritsResource('video', 'media_library'), 'Video should inherit from media_library');
    });
  });

  describe('Allow and Deny Rules', () => {
    test('should allow specific role on specific resource with specific privilege', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addResource('blog', null)
        .allow(['user'], ['blog'], ['read'])
        .build();

      assert.ok(acl.isAllowed('user', 'blog', 'read'), 'User should be allowed to read blog');
      assert.ok(!acl.isAllowed('user', 'blog', 'write'), 'User should not be allowed to write blog');
    });

    test('should deny specific role on specific resource with specific privilege', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addResource('admin_panel', null)
        .allow(['user'], null, null)
        .deny(['user'], ['admin_panel'], null)
        .build();

      assert.ok(!acl.isAllowed('user', 'admin_panel', 'read'), 'User should be denied access to admin_panel');
    });

    test('should allow all privileges with null privileges parameter', () => {
      const acl = new JsAclBuilder()
        .addRole('admin', null)
        .addResource('blog', null)
        .allow(['admin'], ['blog'], null)
        .build();

      assert.ok(acl.isAllowed('admin', 'blog', 'read'), 'Admin should be allowed to read');
      assert.ok(acl.isAllowed('admin', 'blog', 'write'), 'Admin should be allowed to write');
      assert.ok(acl.isAllowed('admin', 'blog', 'delete'), 'Admin should be allowed to delete');
    });

    test('should allow all resources with null resources parameter', () => {
      const acl = new JsAclBuilder()
        .addRole('admin', null)
        .addResources([['blog', null], ['forum', null]])
        .allow(['admin'], null, ['read'])
        .build();

      assert.ok(acl.isAllowed('admin', 'blog', 'read'), 'Admin should be allowed to read blog');
      assert.ok(acl.isAllowed('admin', 'forum', 'read'), 'Admin should be allowed to read forum');
    });

    test('should allow all roles with null roles parameter', () => {
      const acl = new JsAclBuilder()
        .addRoles([['guest', null], ['user', null]])
        .addResource('homepage', null)
        .allow(null, ['homepage'], ['read'])
        .build();

      assert.ok(acl.isAllowed('guest', 'homepage', 'read'), 'Guest should be allowed to read homepage');
      assert.ok(acl.isAllowed('user', 'homepage', 'read'), 'User should be allowed to read homepage');
    });

    test('should handle method chaining', () => {
      const acl = new JsAclBuilder()
        .addRole('guest', null)
        .addRole('user', ['guest'])
        .addResource('blog', null)
        .allow(['guest'], ['blog'], ['read'])
        .allow(['user'], ['blog'], ['read', 'write'])
        .build();

      assert.ok(acl.isAllowed('guest', 'blog', 'read'), 'Guest should be allowed to read');
      assert.ok(acl.isAllowed('user', 'blog', 'write'), 'User should be allowed to write');
    });
  });

  describe('fromJson', () => {
    test('should create ACL builder from valid JSON', () => {
      const json = JSON.stringify({
        roles: [['guest', null], ['user', ['guest']]],
        resources: [['blog', null]],
        allow: [['blog', [['guest', ['read']]]]]
      });

      const builder = JsAclBuilder.fromJson(json);
      const acl = builder.build();

      assert.ok(acl.hasRole('guest'), 'ACL should have guest role');
      assert.ok(acl.hasRole('user'), 'ACL should have user role');
      assert.ok(acl.hasResource('blog'), 'ACL should have blog resource');
      assert.ok(acl.isAllowed('guest', 'blog', 'read'), 'Guest should be allowed to read blog');
    });

    test('should throw error for invalid JSON', () => {
      assert.throws(
        () => JsAclBuilder.fromJson('invalid json'),
        'Should throw error for invalid JSON'
      );
    });
  });
});

describe('JsAcl', () => {
  describe('Constructor', () => {
    test('should create an empty ACL', () => {
      const acl = new JsAcl();
      assert.ok(acl, 'ACL should be created');
    });
  });

  describe('fromJson', () => {
    test('should create ACL from valid JSON', () => {
      const json = JSON.stringify({
        roles: [['guest', null], ['user', ['guest']]],
        resources: [['blog', null]],
        allow: [['blog', [['guest', ['read']]]]]
      });

      const acl = JsAcl.fromJson(json);
      assert.ok(acl.hasRole('guest'), 'ACL should have guest role');
      assert.ok(acl.isAllowed('guest', 'blog', 'read'), 'Guest should be allowed to read blog');
    });

    test('should throw error for invalid JSON', () => {
      assert.throws(
        () => JsAcl.fromJson('invalid json'),
        'Should throw error for invalid JSON'
      );
    });
  });

  describe('hasRole and hasResource', () => {
    test('should check if role exists', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .build();

      assert.ok(acl.hasRole('user'), 'Should return true for existing role');
      assert.ok(!acl.hasRole('admin'), 'Should return false for non-existing role');
    });

    test('should check if resource exists', () => {
      const acl = new JsAclBuilder()
        .addResource('blog', null)
        .build();

      assert.ok(acl.hasResource('blog'), 'Should return true for existing resource');
      assert.ok(!acl.hasResource('forum'), 'Should return false for non-existing resource');
    });
  });

  describe('inheritsRole and inheritsResource', () => {
    test('should check role inheritance', () => {
      const acl = new JsAclBuilder()
        .addRole('guest', null)
        .addRole('user', ['guest'])
        .addRole('admin', ['user'])
        .build();

      assert.ok(acl.inheritsRole('user', 'guest'), 'User should inherit from guest');
      assert.ok(acl.inheritsRole('admin', 'user'), 'Admin should inherit from user');
      assert.ok(!acl.inheritsRole('guest', 'user'), 'Guest should not inherit from user');
    });

    test('should check resource inheritance', () => {
      const acl = new JsAclBuilder()
        .addResource('public_pages', null)
        .addResource('blog', ['public_pages'])
        .addResource('blog_post', ['blog'])
        .build();

      assert.ok(acl.inheritsResource('blog', 'public_pages'), 'Blog should inherit from public_pages');
      assert.ok(acl.inheritsResource('blog_post', 'blog'), 'Blog_post should inherit from blog');
      assert.ok(!acl.inheritsResource('public_pages', 'blog'), 'Public_pages should not inherit from blog');
    });
  });

  describe('isAllowed', () => {
    test('should check basic permission', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addResource('blog', null)
        .allow(['user'], ['blog'], ['read'])
        .build();

      assert.ok(acl.isAllowed('user', 'blog', 'read'), 'User should be allowed to read blog');
      assert.ok(!acl.isAllowed('user', 'blog', 'write'), 'User should not be allowed to write blog');
    });

    test('should respect role inheritance for permissions', () => {
      const acl = new JsAclBuilder()
        .addRole('guest', null)
        .addRole('user', ['guest'])
        .addResource('blog', null)
        .allow(['guest'], ['blog'], ['read'])
        .build();

      assert.ok(acl.isAllowed('guest', 'blog', 'read'), 'Guest should be allowed to read');
      assert.ok(acl.isAllowed('user', 'blog', 'read'), 'User should inherit read permission from guest');
    });

    test('should respect resource inheritance for permissions', () => {
      const acl = new JsAclBuilder()
        .addRole('guest', null)
        .addResource('public_pages', null)
        .addResource('blog', ['public_pages'])
        .allow(['guest'], ['public_pages'], ['read'])
        .build();

      assert.ok(acl.isAllowed('guest', 'public_pages', 'read'), 'Guest should be allowed to read public_pages');
      assert.ok(acl.isAllowed('guest', 'blog', 'read'), 'Guest should be allowed to read blog (inherited)');
    });

    test('should handle null role (all roles)', () => {
      const acl = new JsAclBuilder()
        .addRoles([['guest', null], ['user', null]])
        .addResource('homepage', null)
        .allow(null, ['homepage'], ['read'])
        .build();

      assert.ok(acl.isAllowed(null, 'homepage', 'read'), 'All roles should be allowed');
      assert.ok(acl.isAllowed('guest', 'homepage', 'read'), 'Guest should be allowed');
      assert.ok(acl.isAllowed('user', 'homepage', 'read'), 'User should be allowed');
    });

    test('should handle null resource (all resources)', () => {
      const acl = new JsAclBuilder()
        .addRole('admin', null)
        .addResources([['blog', null], ['forum', null]])
        .allow(['admin'], null, ['read'])
        .build();

      assert.ok(acl.isAllowed('admin', null, 'read'), 'Admin should be allowed on all resources');
      assert.ok(acl.isAllowed('admin', 'blog', 'read'), 'Admin should be allowed on blog');
      assert.ok(acl.isAllowed('admin', 'forum', 'read'), 'Admin should be allowed on forum');
    });

    test('should handle null privilege (all privileges)', () => {
      const acl = new JsAclBuilder()
        .addRole('admin', null)
        .addResource('blog', null)
        .allow(['admin'], ['blog'], null)
        .build();

      assert.ok(acl.isAllowed('admin', 'blog', null), 'Admin should be allowed all privileges');
      assert.ok(acl.isAllowed('admin', 'blog', 'read'), 'Admin should be allowed to read');
      assert.ok(acl.isAllowed('admin', 'blog', 'write'), 'Admin should be allowed to write');
      assert.ok(acl.isAllowed('admin', 'blog', 'delete'), 'Admin should be allowed to delete');
    });

    test('should handle deny rules overriding allow rules', () => {
      const acl = new JsAclBuilder()
        .addRole('editor', null)
        .addResource('blog', null)
        .addResource('admin_panel', null)
        .allow(['editor'], ['blog'], null)  // Allow blog specifically
        .deny(['editor'], ['admin_panel'], null)  // Deny admin_panel
        .build();

      assert.ok(acl.isAllowed('editor', 'blog', 'read'), 'Editor can access blog');
      assert.ok(!acl.isAllowed('editor', 'admin_panel', 'read'), 'Editor should be denied admin_panel access');
    });
  });

  describe('isAllowedAny', () => {
    test('should check if any role has permission', () => {
      const acl = new JsAclBuilder()
        .addRoles([['guest', null], ['user', null], ['admin', null]])
        .addResource('blog', null)
        .allow(['admin'], ['blog'], ['delete'])
        .build();

      assert.ok(
        acl.isAllowedAny(['guest', 'user', 'admin'], ['blog'], ['delete']),
        'Should return true if any role has permission'
      );
      assert.ok(
        !acl.isAllowedAny(['guest', 'user'], ['blog'], ['delete']),
        'Should return false if no role has permission'
      );
    });

    test('should check if any resource is accessible', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addResources([['blog', null], ['forum', null], ['admin_panel', null]])
        .allow(['user'], ['blog'], ['read'])
        .build();

      assert.ok(
        acl.isAllowedAny(['user'], ['blog', 'forum', 'admin_panel'], ['read']),
        'Should return true if any resource is accessible'
      );
      assert.ok(
        !acl.isAllowedAny(['user'], ['forum', 'admin_panel'], ['read']),
        'Should return false if no resource is accessible'
      );
    });

    test('should check if any privilege is allowed', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addResource('blog', null)
        .allow(['user'], ['blog'], ['read'])
        .build();

      assert.ok(
        acl.isAllowedAny(['user'], ['blog'], ['read', 'write', 'delete']),
        'Should return true if any privilege is allowed'
      );
      assert.ok(
        !acl.isAllowedAny(['user'], ['blog'], ['write', 'delete']),
        'Should return false if no privilege is allowed'
      );
    });

    test('should handle null parameters', () => {
      const acl = new JsAclBuilder()
        .addRoles([['guest', null], ['user', null]])
        .addResource('homepage', null)
        .allow(null, ['homepage'], ['read'])
        .build();

      assert.ok(
        acl.isAllowedAny(null, ['homepage'], ['read']),
        'Should work with null roles'
      );
    });
  });
});

describe('Convenience Functions', () => {
  test('createAclFromJson should create ACL from JSON', () => {
    const json = JSON.stringify({
      roles: [['guest', null]],
      resources: [['blog', null]],
      allow: [['blog', [['guest', ['read']]]]]
    });

    const acl = createAclFromJson(json);
    assert.ok(acl instanceof JsAcl, 'Should return JsAcl instance');
    assert.ok(acl.isAllowed('guest', 'blog', 'read'), 'Should have correct permissions');
  });

  test('checkPermission should check permission directly', () => {
    const json = JSON.stringify({
      roles: [['guest', null]],
      resources: [['blog', null]],
      allow: [['blog', [['guest', ['read']]]]]
    });

    assert.ok(
      checkPermission(json, 'guest', 'blog', 'read'),
      'Guest should be allowed to read blog'
    );
    assert.ok(
      !checkPermission(json, 'guest', 'blog', 'write'),
      'Guest should not be allowed to write blog'
    );
  });
});

describe('Extensive ACL Fixture Tests', () => {
  let acl;

  before(() => {
    acl = JsAcl.fromJson(extensiveAclJson);
  });

  describe('Role Hierarchy', () => {
    test('should have all roles defined', () => {
      const roles = ['guest', 'authenticated', 'subscriber', 'contributor', 'author',
                     'editor', 'moderator', 'administrator', 'super_admin'];
      roles.forEach(role => {
        assert.ok(acl.hasRole(role), `ACL should have ${role} role`);
      });
    });

    test('should respect role inheritance chain', () => {
      assert.ok(acl.inheritsRole('authenticated', 'guest'), 'authenticated should inherit from guest');
      assert.ok(acl.inheritsRole('subscriber', 'authenticated'), 'subscriber should inherit from authenticated');
      assert.ok(acl.inheritsRole('author', 'contributor'), 'author should inherit from contributor');
      assert.ok(acl.inheritsRole('administrator', 'moderator'), 'administrator should inherit from moderator');
    });

    test('should handle multiple inheritance', () => {
      assert.ok(acl.hasRole('power_user'), 'ACL should have power_user role');
      assert.ok(acl.hasRole('content_creator'), 'ACL should have content_creator role');
      assert.ok(acl.hasRole('site_admin'), 'ACL should have site_admin role');
    });
  });

  describe('Resource Hierarchy', () => {
    test('should have all top-level resources', () => {
      const resources = ['homepage', 'about', 'contact', 'public_pages', 'admin_panel',
                        'api', 'marketing', 'sales', 'support', 'development', 'hr', 'finance'];
      resources.forEach(resource => {
        assert.ok(acl.hasResource(resource), `ACL should have ${resource} resource`);
      });
    });

    test('should respect resource inheritance', () => {
      assert.ok(acl.inheritsResource('blog', 'public_pages'), 'blog should inherit from public_pages');
      assert.ok(acl.inheritsResource('blog_post', 'blog'), 'blog_post should inherit from blog');
      assert.ok(acl.inheritsResource('api_v1', 'api'), 'api_v1 should inherit from api');
    });

    test('should have deeply nested resources', () => {
      assert.ok(acl.hasResource('blog_comment'), 'ACL should have blog_comment resource');
      assert.ok(acl.hasResource('forum_thread'), 'ACL should have forum_thread resource');
      assert.ok(acl.hasResource('admin_dashboard'), 'ACL should have admin_dashboard resource');
    });
  });

  describe('Public Access Permissions', () => {
    test('guest should have access to public pages', () => {
      assert.ok(acl.isAllowed('guest', 'homepage', null), 'Guest should have full access to homepage');
      assert.ok(acl.isAllowed('guest', 'about', null), 'Guest should have full access to about');
      assert.ok(acl.isAllowed('guest', 'blog', 'read'), 'Guest should be able to read blog');
    });

    test('guest should be able to read public resources', () => {
      assert.ok(acl.isAllowed('guest', 'public_pages', 'read'), 'Guest should read public_pages');
      assert.ok(acl.isAllowed('guest', 'forum', 'read'), 'Guest should read forum');
      assert.ok(acl.isAllowed('guest', 'wiki', 'read'), 'Guest should read wiki');
    });

    test('guest should not have write access to most resources', () => {
      assert.ok(!acl.isAllowed('guest', 'blog', 'write'), 'Guest should not write to blog');
      assert.ok(!acl.isAllowed('guest', 'forum', 'create'), 'Guest should not create forum posts');
    });
  });

  describe('Authenticated User Permissions', () => {
    test('authenticated users should inherit guest permissions', () => {
      assert.ok(acl.isAllowed('authenticated', 'homepage', null), 'Authenticated should access homepage');
      assert.ok(acl.isAllowed('authenticated', 'blog', 'read'), 'Authenticated should read blog');
    });

    test('authenticated users should have additional permissions', () => {
      assert.ok(acl.isAllowed('authenticated', 'contact', 'write'), 'Authenticated can write to contact');
      assert.ok(acl.isAllowed('authenticated', 'blog', 'comment'), 'Authenticated can comment on blog');
      assert.ok(acl.isAllowed('authenticated', 'user_profile', 'edit_own'), 'Authenticated can edit own profile');
    });

    test('authenticated users should be able to create forum content', () => {
      assert.ok(acl.isAllowed('authenticated', 'forum', 'create'), 'Authenticated can create forum posts');
      assert.ok(acl.isAllowed('authenticated', 'forum', 'reply'), 'Authenticated can reply to forum posts');
    });
  });

  describe('Content Creator Permissions', () => {
    test('contributors should be able to create content', () => {
      assert.ok(acl.isAllowed('contributor', 'blog', 'create'), 'Contributor can create blog posts');
      assert.ok(acl.isAllowed('contributor', 'wiki', 'edit'), 'Contributor can edit wiki');
    });

    test('authors should have enhanced permissions', () => {
      assert.ok(acl.isAllowed('author', 'blog_post', 'create'), 'Author can create blog posts');
      assert.ok(acl.isAllowed('author', 'blog_post', 'edit_own'), 'Author can edit own blog posts');
      assert.ok(acl.isAllowed('author', 'blog_post', 'delete_own'), 'Author can delete own blog posts');
      assert.ok(acl.isAllowed('author', 'media_library', 'upload'), 'Author can upload to media library');
    });

    test('editors should have editorial control', () => {
      assert.ok(acl.isAllowed('editor', 'blog', 'edit'), 'Editor can edit blog');
      assert.ok(acl.isAllowed('editor', 'blog', 'delete'), 'Editor can delete blog posts');
      assert.ok(acl.isAllowed('editor', 'blog_post', 'publish'), 'Editor can publish blog posts');
      assert.ok(acl.isAllowed('editor', 'media_library', 'organize'), 'Editor can organize media library');
    });
  });

  describe('Moderator Permissions', () => {
    test('moderators should have moderation powers', () => {
      assert.ok(acl.isAllowed('moderator', 'blog', 'publish'), 'Moderator can publish blog');
      assert.ok(acl.isAllowed('moderator', 'blog_comment', 'approve'), 'Moderator can approve comments');
      assert.ok(acl.isAllowed('moderator', 'blog_comment', 'edit'), 'Moderator can edit comments');
      assert.ok(acl.isAllowed('moderator', 'blog_comment', 'delete'), 'Moderator can delete comments');
    });

    test('moderators should control forum', () => {
      assert.ok(acl.isAllowed('moderator', 'forum', 'lock'), 'Moderator can lock forum threads');
      assert.ok(acl.isAllowed('moderator', 'forum', 'pin'), 'Moderator can pin forum threads');
      assert.ok(acl.isAllowed('moderator', 'forum_thread', 'move'), 'Moderator can move threads');
    });

    test('moderators should have read access to admin dashboard', () => {
      assert.ok(acl.isAllowed('moderator', 'admin_dashboard', 'read'), 'Moderator can read admin dashboard');
    });

    test('moderators should be denied certain admin privileges', () => {
      assert.ok(!acl.isAllowed('moderator', 'admin_panel', 'edit'), 'Moderator cannot edit admin panel');
      assert.ok(!acl.isAllowed('moderator', 'admin_panel', 'delete'), 'Moderator cannot delete from admin panel');
    });
  });

  describe('Administrator Permissions', () => {
    test('administrators should have full admin panel access', () => {
      assert.ok(acl.isAllowed('administrator', 'admin_panel', null), 'Administrator has full admin panel access');
      assert.ok(acl.isAllowed('administrator', 'admin_dashboard', 'customize'), 'Administrator can customize dashboard');
      assert.ok(acl.isAllowed('administrator', 'admin_users', 'create'), 'Administrator can create users');
      assert.ok(acl.isAllowed('administrator', 'admin_users', 'suspend'), 'Administrator can suspend users');
    });

    test('administrators should manage roles and permissions', () => {
      assert.ok(acl.isAllowed('administrator', 'admin_roles', 'create'), 'Administrator can create roles');
      assert.ok(acl.isAllowed('administrator', 'admin_roles', 'edit'), 'Administrator can edit roles');
      assert.ok(acl.isAllowed('administrator', 'admin_roles', 'delete'), 'Administrator can delete roles');
    });

    test('administrators should have report access', () => {
      assert.ok(acl.isAllowed('administrator', 'reports', 'generate'), 'Administrator can generate reports');
      assert.ok(acl.isAllowed('administrator', 'reports', 'schedule'), 'Administrator can schedule reports');
      assert.ok(acl.isAllowed('administrator', 'reports', 'export'), 'Administrator can export reports');
    });

    test('administrators should be denied certain system operations', () => {
      assert.ok(!acl.isAllowed('administrator', 'admin_system', 'delete'), 'Administrator cannot delete from system');
      assert.ok(!acl.isAllowed('administrator', 'admin_system', 'reset'), 'Administrator cannot reset system');
    });
  });

  describe('Super Admin Permissions', () => {
    test('super admin should have system-level access', () => {
      assert.ok(acl.isAllowed('super_admin', 'admin_system', 'read'), 'Super admin can read system');
      assert.ok(acl.isAllowed('super_admin', 'admin_system', 'configure'), 'Super admin can configure system');
      assert.ok(acl.isAllowed('super_admin', 'admin_system', 'maintenance'), 'Super admin can do maintenance');
    });

    test('super admin should have advanced settings access', () => {
      assert.ok(acl.isAllowed('super_admin', 'admin_settings', 'advanced'), 'Super admin can access advanced settings');
    });

    test('super admin should manage logs', () => {
      assert.ok(acl.isAllowed('super_admin', 'admin_logs', 'export'), 'Super admin can export logs');
      assert.ok(acl.isAllowed('super_admin', 'admin_logs', 'delete'), 'Super admin can delete logs');
    });
  });

  describe('Department-Specific Permissions', () => {
    test('marketing team should have marketing access', () => {
      assert.ok(acl.isAllowed('marketing_viewer', 'marketing', 'read'), 'Marketing viewer can read');
      assert.ok(acl.isAllowed('marketing_editor', 'marketing_campaign', 'create'), 'Marketing editor can create campaigns');
      assert.ok(acl.isAllowed('marketing_manager', 'marketing_campaign', 'launch'), 'Marketing manager can launch campaigns');
    });

    test('sales team should have sales access', () => {
      assert.ok(acl.isAllowed('sales_rep', 'sales_leads', 'create'), 'Sales rep can create leads');
      assert.ok(acl.isAllowed('sales_rep', 'sales_leads', 'edit_own'), 'Sales rep can edit own leads');
      assert.ok(acl.isAllowed('sales_manager', 'sales_leads', 'assign'), 'Sales manager can assign leads');
      assert.ok(!acl.isAllowed('sales_rep', 'sales_orders', 'approve'), 'Sales rep cannot approve orders');
    });

    test('support team should have tiered access', () => {
      assert.ok(acl.isAllowed('support_tier1', 'support_ticket', 'create'), 'Tier 1 can create tickets');
      assert.ok(acl.isAllowed('support_tier2', 'support', 'escalate'), 'Tier 2 can escalate');
      assert.ok(acl.isAllowed('support_tier3', 'support', 'priority'), 'Tier 3 can set priority');
      assert.ok(acl.isAllowed('support_manager', 'support_ticket', 'assign'), 'Manager can assign tickets');
    });

    test('development team should have dev access', () => {
      assert.ok(acl.isAllowed('developer', 'dev_repository', 'commit'), 'Developer can commit');
      assert.ok(acl.isAllowed('senior_developer', 'development', 'review'), 'Senior dev can review');
      assert.ok(acl.isAllowed('tech_lead', 'dev_deployment', 'deploy_staging'), 'Tech lead can deploy to staging');
      assert.ok(!acl.isAllowed('developer', 'dev_deployment', 'deploy_production'), 'Developer cannot deploy to production');
    });

    test('HR team should have HR access', () => {
      assert.ok(acl.isAllowed('hr_coordinator', 'hr', 'create'), 'HR coordinator can create');
      assert.ok(acl.isAllowed('hr_manager', 'hr_payroll', 'process'), 'HR manager can process payroll');
      assert.ok(!acl.isAllowed('hr_coordinator', 'hr_payroll', 'process'), 'HR coordinator cannot process payroll');
      assert.ok(acl.isAllowed('hr_director', 'hr', null), 'HR director has full access');
    });

    test('finance team should have finance access', () => {
      assert.ok(acl.isAllowed('finance_clerk', 'finance', 'create'), 'Finance clerk can create');
      assert.ok(acl.isAllowed('accountant', 'finance_accounting', 'reconcile'), 'Accountant can reconcile');
      assert.ok(acl.isAllowed('finance_manager', 'finance', 'approve'), 'Finance manager can approve');
      assert.ok(acl.isAllowed('cfo', 'report_financial', null), 'CFO has full financial report access');
    });
  });

  describe('API Access', () => {
    test('API users should have API access', () => {
      assert.ok(acl.isAllowed('api_user', 'api_public', 'read'), 'API user can read public API');
      assert.ok(acl.isAllowed('api_user', 'api_public', 'write'), 'API user can write to public API');
      assert.ok(acl.isAllowed('api_user', 'api_private', 'read'), 'API user can read private API');
    });

    test('API admin should have full API access', () => {
      assert.ok(acl.isAllowed('api_admin', 'api_private', null), 'API admin has full private API access');
      assert.ok(acl.isAllowed('api_admin', 'api_admin', null), 'API admin has full admin API access');
    });
  });

  describe('Analytics and Reporting', () => {
    test('analysts should have analytics access', () => {
      assert.ok(acl.isAllowed('analyst', 'report_analytics', 'read'), 'Analyst can read analytics');
      assert.ok(acl.isAllowed('data_analyst', 'report_analytics', 'generate'), 'Data analyst can generate reports');
      assert.ok(acl.isAllowed('analytics_manager', 'report_analytics', 'customize'), 'Analytics manager can customize');
    });
  });

  describe('Deny Rules', () => {
    test('editors should be denied admin panel access', () => {
      assert.ok(!acl.isAllowed('editor', 'admin_panel', 'read'), 'Editor should be denied admin panel');
      assert.ok(!acl.isAllowed('editor', 'admin_panel', 'write'), 'Editor should be denied admin panel write');
    });

    test('content roles should be denied finance access', () => {
      assert.ok(!acl.isAllowed('contributor', 'finance', 'read'), 'Contributor denied finance access');
      assert.ok(!acl.isAllowed('author', 'finance', 'read'), 'Author denied finance access');
      assert.ok(!acl.isAllowed('editor', 'finance', 'read'), 'Editor denied finance access');
      assert.ok(!acl.isAllowed('moderator', 'finance', 'read'), 'Moderator denied finance access');
    });

    test('moderators should not access private user data', () => {
      assert.ok(!acl.isAllowed('moderator', 'user_private_data', 'read'), 'Moderator denied private user data');
    });
  });

  describe('isAllowedAny with Extensive ACL', () => {
    test('should check if any role from team has permission', () => {
      assert.ok(
        acl.isAllowedAny(['sales_rep', 'sales_manager'], ['sales_leads'], ['assign']),
        'Sales manager from team can assign leads'
      );
    });

    test('should check if user can access any resource', () => {
      assert.ok(
        acl.isAllowedAny(['developer'], ['dev_repository', 'dev_deployment', 'dev_monitoring'], ['read']),
        'Developer can read at least one dev resource'
      );
    });

    test('should check if user has any of multiple privileges', () => {
      assert.ok(
        acl.isAllowedAny(['support_tier1'], ['support_ticket'], ['create', 'resolve', 'escalate']),
        'Tier 1 support has at least one of these privileges'
      );
    });
  });
});

describe('Complex Scenarios', () => {
  test('should handle multi-level role inheritance', () => {
    const acl = new JsAclBuilder()
      .addRole('guest', null)
      .addRole('authenticated', ['guest'])
      .addRole('subscriber', ['authenticated'])
      .addRole('contributor', ['subscriber'])
      .addRole('author', ['contributor'])
      .addResource('blog', null)
      .allow(['guest'], ['blog'], ['read'])
      .allow(['contributor'], ['blog'], ['create'])
      .build();

    // Author should inherit permissions from all parent roles
    assert.ok(acl.isAllowed('author', 'blog', 'read'), 'Author should inherit read from guest');
    assert.ok(acl.isAllowed('author', 'blog', 'create'), 'Author should inherit create from contributor');
  });

  test('should handle multi-level resource inheritance', () => {
    const acl = new JsAclBuilder()
      .addRole('user', null)
      .addResource('public_pages', null)
      .addResource('blog', ['public_pages'])
      .addResource('blog_post', ['blog'])
      .addResource('blog_comment', ['blog_post'])
      .allow(['user'], ['public_pages'], ['read'])
      .build();

    // Deep resources should inherit permissions
    assert.ok(acl.isAllowed('user', 'blog_comment', 'read'), 'Should inherit read through chain');
  });

  test('should handle multiple inheritance paths', () => {
    const acl = new JsAclBuilder()
      .addRoles([
        ['subscriber', null],
        ['commenter', null],
        ['power_user', ['subscriber', 'commenter']]
      ])
      .addResource('blog', null)
      .allow(['subscriber'], ['blog'], ['read'])
      .allow(['commenter'], ['blog'], ['comment'])
      .build();

    assert.ok(acl.isAllowed('power_user', 'blog', 'read'), 'Power user should inherit read from subscriber');
    assert.ok(acl.isAllowed('power_user', 'blog', 'comment'), 'Power user should inherit comment from commenter');
  });

  test('should handle complex allow and deny combinations', () => {
    const acl = new JsAclBuilder()
      .addRole('editor', null)
      .addResources([['blog', null], ['admin_panel', null], ['admin_system', null]])
      .deny(['editor'], ['admin_panel'], null) // Deny admin_panel first
      .allow(['editor'], ['blog'], null) // Allow blog
      .allow(['editor'], ['admin_system'], null) // Allow admin_system
      .build();

    assert.ok(acl.isAllowed('editor', 'blog', 'write'), 'Editor can write to blog');
    assert.ok(!acl.isAllowed('editor', 'admin_panel', 'read'), 'Editor denied admin panel');
    assert.ok(acl.isAllowed('editor', 'admin_system', 'read'), 'Editor can access admin_system');
  });

  test('should handle granular privilege control', () => {
    const acl = new JsAclBuilder()
      .addRole('author', null)
      .addResource('blog_post', null)
      .allow(['author'], ['blog_post'], ['create', 'edit_own', 'delete_own'])
      .build();

    assert.ok(acl.isAllowed('author', 'blog_post', 'create'), 'Author can create');
    assert.ok(acl.isAllowed('author', 'blog_post', 'edit_own'), 'Author can edit own');
    assert.ok(!acl.isAllowed('author', 'blog_post', 'edit'), 'Author cannot edit others');
    assert.ok(!acl.isAllowed('author', 'blog_post', 'delete'), 'Author cannot delete others');
  });
});

describe('Error Handling', () => {
  test('should handle non-existent roles gracefully', () => {
    const acl = new JsAclBuilder()
      .addResource('blog', null)
      .build();

    // Should not throw, just return false
    assert.ok(!acl.isAllowed('non_existent_role', 'blog', 'read'), 'Should return false for non-existent role');
  });

  test('should handle non-existent resources gracefully', () => {
    const acl = new JsAclBuilder()
      .addRole('user', null)
      .build();

    // Should not throw, just return false
    assert.ok(!acl.isAllowed('user', 'non_existent_resource', 'read'), 'Should return false for non-existent resource');
  });

  test('should throw error for invalid JSON structure', () => {
    const invalidJson = JSON.stringify({
      roles: 'invalid', // Should be array
      resources: [],
      allow: []
    });

    assert.throws(
      () => JsAcl.fromJson(invalidJson),
      'Should throw for invalid structure'
    );
  });
});

describe('Memory Management', () => {
  test('should properly dispose of ACL instances', () => {
    const acl = new JsAclBuilder()
      .addRole('user', null)
      .addResource('blog', null)
      .build();

    // Test that disposal works without errors
    assert.doesNotThrow(() => acl.free(), 'Should free without errors');
  });

  test('should handle Symbol.dispose for builder', () => {
    const builder = new JsAclBuilder();
    assert.ok(typeof builder[Symbol.dispose] === 'function', 'Builder should have Symbol.dispose');
  });

  test('should handle Symbol.dispose for ACL', () => {
    const acl = new JsAcl();
    assert.ok(typeof acl[Symbol.dispose] === 'function', 'ACL should have Symbol.dispose');
  });
});

