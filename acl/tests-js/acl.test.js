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
    });

    test('should add a single resource', () => {
      const acl = new JsAclBuilder()
        .addResource('blog', null)
        .build();

      assert.ok(acl.hasResource('blog'), 'ACL should have the blog resource');
    });

    test('should add a resource with parent', () => {
      const acl = new JsAclBuilder()
        .addResource('blog', null)
        .addResource('post', ['blog'])
        .build();

      assert.ok(acl.hasResource('post'), 'ACL should have the post resource');
      assert.ok(acl.inheritsResource('post', 'blog'), 'Post should inherit from blog');
    });

    test('should add multiple resources at once', () => {
      const acl = new JsAclBuilder()
        .addResources([
          ['blog', null],
          ['post', ['blog']],
          ['comment', ['post']]
        ])
        .build();

      assert.ok(acl.hasResource('blog'), 'ACL should have blog resource');
      assert.ok(acl.hasResource('post'), 'ACL should have post resource');
      assert.ok(acl.hasResource('comment'), 'ACL should have comment resource');
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
      assert.ok(!acl.isAllowed('user', 'blog', 'write'), 'User should not be allowed to write to blog');
    });

    test('should deny specific role on specific resource with specific privilege', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addResource('admin_panel', null)
        .allow(['user'], ['admin_panel'], ['read'])
        .deny(['user'], ['admin_panel'], ['delete'])
        .build();

      assert.ok(acl.isAllowed('user', 'admin_panel', 'read'), 'User should be allowed to read');
      assert.ok(!acl.isAllowed('user', 'admin_panel', 'delete'), 'User should be denied delete');
    });

    test('should allow all privileges with null privileges parameter', () => {
      const acl = new JsAclBuilder()
        .addRole('admin', null)
        .addResource('blog', null)
        .allow(['admin'], ['blog'], null)
        .build();

      assert.ok(acl.isAllowed('admin', 'blog', 'read'), 'Admin should have read access');
      assert.ok(acl.isAllowed('admin', 'blog', 'write'), 'Admin should have write access');
      assert.ok(acl.isAllowed('admin', 'blog', 'delete'), 'Admin should have delete access');
    });

    test('should allow all resources with null resources parameter', () => {
      const acl = new JsAclBuilder()
        .addRole('admin', null)
        .addResource('blog', null)
        .addResource('forum', null)
        .allow(['admin'], null, ['read'])
        .build();

      assert.ok(acl.isAllowed('admin', 'blog', 'read'), 'Admin should read blog');
      assert.ok(acl.isAllowed('admin', 'forum', 'read'), 'Admin should read forum');
    });

    test('should allow all roles with null roles parameter', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addRole('guest', null)
        .addResource('homepage', null)
        .allow(null, ['homepage'], ['read'])
        .build();

      assert.ok(acl.isAllowed('user', 'homepage', 'read'), 'User should read homepage');
      assert.ok(acl.isAllowed('guest', 'homepage', 'read'), 'Guest should read homepage');
    });

    test('should handle method chaining', () => {
      const acl = new JsAclBuilder()
        .addRole('guest', null)
        .addRole('user', ['guest'])
        .addResource('blog', null)
        .allow(['guest'], ['blog'], ['read'])
        .allow(['user'], ['blog'], ['write'])
        .build();

      assert.ok(acl.isAllowed('guest', 'blog', 'read'), 'Guest should read');
      assert.ok(acl.isAllowed('user', 'blog', 'write'), 'User should write');
    });
  });

  describe('fromJson', () => {
    test('should create ACL builder from valid JSON', () => {
      const builder = JsAclBuilder.fromJson(extensiveAclJson);
      assert.ok(builder instanceof JsAclBuilder, 'Should create builder instance');

      const acl = builder.build();
      assert.ok(acl instanceof JsAcl, 'Should build ACL instance');
      assert.ok(acl.hasRole('guest'), 'Should have guest role from JSON');
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
      const builder = new JsAclBuilder();
      const acl = builder.build();
      assert.ok(acl instanceof JsAcl, 'Should create ACL instance');
    });
  });

  describe('fromJson', () => {
    test('should create ACL from valid JSON', () => {
      const acl = JsAcl.fromJson(extensiveAclJson);
      assert.ok(acl instanceof JsAcl, 'Should create ACL instance');
      assert.ok(acl.hasRole('guest'), 'Should have guest role');
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

      assert.ok(acl.hasRole('user'), 'Should have user role');
      assert.ok(!acl.hasRole('admin'), 'Should not have admin role');
    });

    test('should check if resource exists', () => {
      const acl = new JsAclBuilder()
        .addResource('blog', null)
        .build();

      assert.ok(acl.hasResource('blog'), 'Should have blog resource');
      assert.ok(!acl.hasResource('forum'), 'Should not have forum resource');
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
      assert.ok(acl.inheritsRole('admin', 'guest'), 'Admin should inherit from guest (transitive)');
    });

    test('should check resource inheritance', () => {
      const acl = new JsAclBuilder()
        .addResource('blog', null)
        .addResource('post', ['blog'])
        .addResource('comment', ['post'])
        .build();

      assert.ok(acl.inheritsResource('post', 'blog'), 'Post should inherit from blog');
      assert.ok(acl.inheritsResource('comment', 'post'), 'Comment should inherit from post');
      assert.ok(acl.inheritsResource('comment', 'blog'), 'Comment should inherit from blog (transitive)');
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
      assert.ok(!acl.isAllowed('user', 'blog', 'write'), 'User should not be allowed to write to blog');
    });

    test('should respect role inheritance for permissions', () => {
      const acl = new JsAclBuilder()
        .addRole('guest', null)
        .addRole('user', ['guest'])
        .addResource('blog', null)
        .allow(['guest'], ['blog'], ['read'])
        .build();

      assert.ok(acl.isAllowed('user', 'blog', 'read'), 'User should inherit read permission from guest');
    });

    test('should respect resource inheritance for permissions', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addResource('blog', null)
        .addResource('post', ['blog'])
        .allow(['user'], ['blog'], ['read'])
        .build();

      assert.ok(acl.isAllowed('user', 'post', 'read'), 'User should have read permission on child resource');
    });

    test('should handle null role (all roles)', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addResource('homepage', null)
        .allow(null, ['homepage'], ['read'])
        .build();

      assert.ok(acl.isAllowed(null, 'homepage', 'read'), 'All roles should read homepage');
    });

    test('should handle null resource (all resources)', () => {
      const acl = new JsAclBuilder()
        .addRole('admin', null)
        .addResource('blog', null)
        .allow(['admin'], null, ['read'])
        .build();

      assert.ok(acl.isAllowed('admin', null, 'read'), 'Admin should read all resources');
    });

    test('should handle null privilege (all privileges)', () => {
      const acl = new JsAclBuilder()
        .addRole('admin', null)
        .addResource('blog', null)
        .allow(['admin'], ['blog'], null)
        .build();

      assert.ok(acl.isAllowed('admin', 'blog', null), 'Admin should have all privileges on blog');
    });

    test('should handle deny rules overriding allow rules', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addResource('admin_panel', null)
        .allow(['user'], ['admin_panel'], ['read', 'write'])
        .deny(['user'], ['admin_panel'], ['write'])
        .build();

      assert.ok(acl.isAllowed('user', 'admin_panel', 'read'), 'User should be allowed to read');
      assert.ok(!acl.isAllowed('user', 'admin_panel', 'write'), 'User should be denied write');
    });
  });

  describe('isAllowedAny', () => {
    test('should check if any role has permission', () => {
      const acl = new JsAclBuilder()
        .addRole('guest', null)
        .addRole('user', null)
        .addResource('blog', null)
        .allow(['user'], ['blog'], ['write'])
        .build();

      assert.ok(acl.isAllowedAny(['guest', 'user'], ['blog'], ['write']), 'At least one role should have permission');
      assert.ok(!acl.isAllowedAny(['guest'], ['blog'], ['write']), 'Guest alone should not have permission');
    });

    test('should check if any resource is accessible', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addResource('blog', null)
        .addResource('forum', null)
        .allow(['user'], ['forum'], ['read'])
        .build();

      assert.ok(acl.isAllowedAny(['user'], ['blog', 'forum'], ['read']), 'At least one resource should be accessible');
    });

    test('should check if any privilege is allowed', () => {
      const acl = new JsAclBuilder()
        .addRole('user', null)
        .addResource('blog', null)
        .allow(['user'], ['blog'], ['read'])
        .build();

      assert.ok(acl.isAllowedAny(['user'], ['blog'], ['read', 'write']), 'At least one privilege should be allowed');
    });

    test('should handle null parameters', () => {
      const acl = new JsAclBuilder()
        .addRole('admin', null)
        .addResource('blog', null)
        .allow(['admin'], ['blog'], null)
        .build();

      assert.ok(acl.isAllowedAny(['admin'], ['blog'], null), 'Should work with null privileges');
    });
  });
});

describe('Convenience Functions', () => {
  test('createAclFromJson should create ACL from JSON', () => {
    const acl = createAclFromJson(extensiveAclJson);
    assert.ok(acl instanceof JsAcl, 'Should create ACL instance');
    assert.ok(acl.hasRole('guest'), 'Should have guest role');
  });

  test('checkPermission should check permission directly', () => {
    const aclJson = JSON.stringify({
      roles: [['user', null]],
      resources: [['blog', null]],
      allow: [['blog', [['user', ['read']]]]]
    });

    assert.ok(checkPermission(aclJson, 'user', 'blog', 'read'), 'Should allow user to read blog');
    assert.ok(!checkPermission(aclJson, 'user', 'blog', 'write'), 'Should not allow user to write to blog');
  });
});

describe('Extensive ACL Fixture Tests', () => {
  let acl;

  before(() => {
    acl = JsAcl.fromJson(extensiveAclJson);
  });

  describe('Role Hierarchy', () => {
    test('should have all roles defined', () => {
      const expectedRoles = [
        'guest', 'authenticated', 'subscriber', 'contributor', 'author', 'editor',
        'moderator', 'administrator', 'super_admin', 'power_user', 'api_user',
        'api_admin', 'support_tier1', 'support_tier2', 'support_manager',
        'developer', 'tech_lead', 'analyst', 'finance_manager', 'cfo'
      ];

      for (const role of expectedRoles) {
        assert.ok(acl.hasRole(role), `ACL should have ${role} role`);
      }
    });

    test('should respect role inheritance chain', () => {
      assert.ok(acl.inheritsRole('authenticated', 'guest'), 'authenticated should inherit from guest');
      assert.ok(acl.inheritsRole('subscriber', 'authenticated'), 'subscriber should inherit from authenticated');
      assert.ok(acl.inheritsRole('contributor', 'subscriber'), 'contributor should inherit from subscriber');
      assert.ok(acl.inheritsRole('author', 'contributor'), 'author should inherit from contributor');
      assert.ok(acl.inheritsRole('editor', 'author'), 'editor should inherit from author');
      assert.ok(acl.inheritsRole('moderator', 'editor'), 'moderator should inherit from editor');
      assert.ok(acl.inheritsRole('administrator', 'moderator'), 'administrator should inherit from moderator');
      assert.ok(acl.inheritsRole('super_admin', 'administrator'), 'super_admin should inherit from administrator');
    });

    test('should handle multiple inheritance', () => {
      assert.ok(acl.inheritsRole('power_user', 'subscriber'), 'power_user should inherit from subscriber');
      assert.ok(acl.inheritsRole('power_user', 'moderator'), 'power_user should inherit from moderator');
      assert.ok(acl.inheritsRole('support_manager', 'support_tier2'), 'support_manager should inherit from support_tier2');
      assert.ok(acl.inheritsRole('support_manager', 'moderator'), 'support_manager should inherit from moderator');
      assert.ok(acl.inheritsRole('tech_lead', 'developer'), 'tech_lead should inherit from developer');
      assert.ok(acl.inheritsRole('tech_lead', 'moderator'), 'tech_lead should inherit from moderator');
      assert.ok(acl.inheritsRole('cfo', 'finance_manager'), 'cfo should inherit from finance_manager');
      assert.ok(acl.inheritsRole('cfo', 'administrator'), 'cfo should inherit from administrator');
    });
  });

  describe('Resource Hierarchy', () => {
    test('should have all top-level resources', () => {
      const topLevelResources = [
        'homepage', 'public_pages', 'user_profile', 'media_library',
        'admin_panel', 'api', 'reports', 'support', 'development', 'finance'
      ];

      for (const resource of topLevelResources) {
        assert.ok(acl.hasResource(resource), `ACL should have ${resource} resource`);
      }
    });

    test('should respect resource inheritance', () => {
      assert.ok(acl.inheritsResource('blog', 'public_pages'), 'blog should inherit from public_pages');
      assert.ok(acl.inheritsResource('blog_post', 'blog'), 'blog_post should inherit from blog');
      assert.ok(acl.inheritsResource('blog_comment', 'blog_post'), 'blog_comment should inherit from blog_post');
      assert.ok(acl.inheritsResource('api_public', 'api'), 'api_public should inherit from api');
      assert.ok(acl.inheritsResource('api_private', 'api'), 'api_private should inherit from api');
    });

    test('should have deeply nested resources', () => {
      assert.ok(acl.hasResource('blog_comment'), 'ACL should have blog_comment resource');
      assert.ok(acl.hasResource('forum_thread'), 'ACL should have forum_thread resource');
      assert.ok(acl.hasResource('admin_settings'), 'ACL should have admin_settings resource');
      assert.ok(acl.hasResource('dev_deployment'), 'ACL should have dev_deployment resource');
    });
  });

  describe('Public Access Permissions', () => {
    test('guest should have access to homepage', () => {
      assert.ok(acl.isAllowed('guest', 'homepage', null), 'Guest should have full access to homepage');
    });

    test('guest should be able to read public resources', () => {
      assert.ok(acl.isAllowed('guest', 'public_pages', 'read'), 'Guest should read public_pages');
      assert.ok(acl.isAllowed('guest', 'blog', 'read'), 'Guest should read blog');
      assert.ok(acl.isAllowed('guest', 'forum', 'read'), 'Guest should read forum');
      assert.ok(acl.isAllowed('guest', 'wiki', 'read'), 'Guest should read wiki');
    });

    test('guest should not have write access to most resources', () => {
      assert.ok(!acl.isAllowed('guest', 'blog', 'write'), 'Guest should not write to blog');
      assert.ok(!acl.isAllowed('guest', 'forum', 'create'), 'Guest should not create forum posts');
      assert.ok(!acl.isAllowed('guest', 'user_profile', 'edit'), 'Guest should not edit user profiles');
    });
  });

  describe('Authenticated User Permissions', () => {
    test('authenticated users should inherit guest permissions', () => {
      assert.ok(acl.isAllowed('authenticated', 'homepage', null), 'Authenticated should access homepage');
      assert.ok(acl.isAllowed('authenticated', 'blog', 'read'), 'Authenticated should read blog');
    });

    test('authenticated users should have additional permissions', () => {
      assert.ok(acl.isAllowed('authenticated', 'blog', 'comment'), 'Authenticated can comment on blog');
      assert.ok(acl.isAllowed('authenticated', 'user_profile', 'edit_own'), 'Authenticated can edit own profile');
      assert.ok(acl.isAllowed('authenticated', 'forum', 'create'), 'Authenticated can create forum content');
    });

    test('authenticated users should manage their own content', () => {
      assert.ok(acl.isAllowed('authenticated', 'blog_comment', 'create'), 'Authenticated can create comments');
      assert.ok(acl.isAllowed('authenticated', 'blog_comment', 'edit_own'), 'Authenticated can edit own comments');
      assert.ok(acl.isAllowed('authenticated', 'user_settings', 'read_own'), 'Authenticated can read own settings');
    });
  });

  describe('Content Creator Permissions', () => {
    test('contributors should be able to create content', () => {
      assert.ok(acl.isAllowed('contributor', 'blog', 'create'), 'Contributor can create blog posts');
      assert.ok(acl.isAllowed('contributor', 'wiki', 'edit'), 'Contributor can edit wiki');
      assert.ok(acl.isAllowed('contributor', 'wiki_page', 'create'), 'Contributor can create wiki pages');
    });

    test('authors should manage their own content', () => {
      assert.ok(acl.isAllowed('author', 'blog_post', 'create'), 'Author can create posts');
      assert.ok(acl.isAllowed('author', 'blog_post', 'edit_own'), 'Author can edit own posts');
      assert.ok(acl.isAllowed('author', 'media_library', 'upload'), 'Author can upload media');
    });

    test('editors should manage all content', () => {
      assert.ok(acl.isAllowed('editor', 'blog', 'delete'), 'Editor can delete blog posts');
      assert.ok(acl.isAllowed('editor', 'blog_post', 'edit'), 'Editor can edit any post');
      assert.ok(acl.isAllowed('editor', 'blog_post', 'publish'), 'Editor can publish posts');
      assert.ok(acl.isAllowed('editor', 'wiki_page', 'delete'), 'Editor can delete wiki pages');
    });
  });

  describe('Moderator Permissions', () => {
    test('moderators should manage user content', () => {
      assert.ok(acl.isAllowed('moderator', 'blog_comment', 'approve'), 'Moderator can approve comments');
      assert.ok(acl.isAllowed('moderator', 'blog_comment', 'edit'), 'Moderator can edit comments');
      assert.ok(acl.isAllowed('moderator', 'forum', 'lock'), 'Moderator can lock forum threads');
      assert.ok(acl.isAllowed('moderator', 'forum_thread', 'move'), 'Moderator can move threads');
    });

    test('moderators should have reporting access', () => {
      assert.ok(acl.isAllowed('moderator', 'reports', 'read'), 'Moderator can read reports');
      assert.ok(acl.isAllowed('moderator', 'reports', 'generate'), 'Moderator can generate reports');
      assert.ok(acl.isAllowed('moderator', 'report_analytics', 'read'), 'Moderator can read analytics');
    });

    test('moderators should edit user profiles', () => {
      assert.ok(acl.isAllowed('moderator', 'user_profile', 'edit'), 'Moderator can edit profiles');
    });
  });

  describe('Administrator Permissions', () => {
    test('administrators should have admin panel access', () => {
      assert.ok(acl.isAllowed('administrator', 'admin_panel', null), 'Administrator has full admin panel access');
      assert.ok(acl.isAllowed('administrator', 'admin_users', 'create'), 'Administrator can create users');
      assert.ok(acl.isAllowed('administrator', 'admin_users', 'delete'), 'Administrator can delete users');
    });

    test('administrators should manage settings', () => {
      assert.ok(acl.isAllowed('administrator', 'admin_settings', 'read'), 'Administrator can read settings');
      assert.ok(acl.isAllowed('administrator', 'admin_settings', 'edit'), 'Administrator can edit settings');
      assert.ok(acl.isAllowed('administrator', 'user_settings', 'edit'), 'Administrator can edit user settings');
    });

    test('administrators should have report access', () => {
      assert.ok(acl.isAllowed('administrator', 'reports', 'generate'), 'Administrator can generate reports');
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
      assert.ok(acl.isAllowed('super_admin', 'admin_settings', 'edit'), 'Super admin can edit settings');
    });

    test('super admin should access private user data', () => {
      assert.ok(acl.isAllowed('super_admin', 'user_private_data', 'read'), 'Super admin can read private data');
    });
  });

  describe('Department-Specific Permissions', () => {
    test('support team should have tiered access', () => {
      assert.ok(acl.isAllowed('support_tier1', 'support_ticket', 'create'), 'Tier 1 can create tickets');
      assert.ok(acl.isAllowed('support_tier1', 'support', 'read'), 'Tier 1 can read support');
      assert.ok(acl.isAllowed('support_tier2', 'support', 'escalate'), 'Tier 2 can escalate');
      assert.ok(acl.isAllowed('support_manager', 'support_ticket', 'assign'), 'Manager can assign tickets');
      assert.ok(!acl.isAllowed('support_tier1', 'support_ticket', 'delete'), 'Tier 1 cannot delete tickets');
    });

    test('development team should have dev access', () => {
      assert.ok(acl.isAllowed('developer', 'dev_repository', 'commit'), 'Developer can commit');
      assert.ok(acl.isAllowed('developer', 'development', 'read'), 'Developer can read development');
      assert.ok(acl.isAllowed('tech_lead', 'dev_deployment', 'deploy_staging'), 'Tech lead can deploy to staging');
      assert.ok(!acl.isAllowed('developer', 'dev_deployment', 'deploy_production'), 'Developer cannot deploy to production');
    });

    test('finance team should have finance access', () => {
      assert.ok(acl.isAllowed('analyst', 'finance', 'read'), 'Analyst can read finance');
      assert.ok(acl.isAllowed('finance_manager', 'finance', 'create'), 'Finance manager can create');
      assert.ok(acl.isAllowed('finance_manager', 'finance_payroll', 'read'), 'Finance manager can read payroll');
      assert.ok(acl.isAllowed('cfo', 'report_financial', null), 'CFO has full financial report access');
      assert.ok(acl.isAllowed('cfo', 'finance', null), 'CFO has full finance access');
    });
  });

  describe('API Access', () => {
    test('API users should have API access', () => {
      assert.ok(acl.isAllowed('api_user', 'api_public', 'read'), 'API user can read public API');
      assert.ok(acl.isAllowed('api_user', 'api_public', 'write'), 'API user can write to public API');
      assert.ok(acl.isAllowed('api_user', 'api_private', 'read'), 'API user can read private API');
      assert.ok(acl.isAllowed('api_user', 'api_private', 'write'), 'API user can write to private API');
    });

    test('API admin should have full API access', () => {
      assert.ok(acl.isAllowed('api_admin', 'api_private', null), 'API admin has full private API access');
      // api_admin inherits from administrator who has admin_panel access, but not direct api resource access
      // so we test what they actually have according to the fixture
      assert.ok(acl.isAllowed('api_admin', 'api_private', 'read'), 'API admin can read private API');
      assert.ok(acl.isAllowed('api_admin', 'api_private', 'write'), 'API admin can write private API');
    });
  });

  describe('Analytics and Reporting', () => {
    test('analysts should have analytics access', () => {
      assert.ok(acl.isAllowed('analyst', 'report_analytics', 'read'), 'Analyst can read analytics');
      assert.ok(acl.isAllowed('analyst', 'report_analytics', 'generate'), 'Analyst can generate reports');
    });

    test('moderators should customize analytics', () => {
      assert.ok(acl.isAllowed('moderator', 'report_analytics', 'customize'), 'Moderator can customize analytics');
    });
  });

  describe('Deny Rules', () => {
    test('editors should be denied admin panel access', () => {
      assert.ok(!acl.isAllowed('editor', 'admin_panel', 'read'), 'Editor should be denied admin panel');
      assert.ok(!acl.isAllowed('editor', 'admin_panel', 'write'), 'Editor should be denied admin panel write');
    });

    test('moderators should have restricted admin panel access', () => {
      assert.ok(!acl.isAllowed('moderator', 'admin_panel', 'edit'), 'Moderator should be denied edit');
      assert.ok(!acl.isAllowed('moderator', 'admin_panel', 'delete'), 'Moderator should be denied delete');
    });

    test('content roles should be denied finance access', () => {
      assert.ok(!acl.isAllowed('contributor', 'finance', 'read'), 'Contributor denied finance access');
      assert.ok(!acl.isAllowed('author', 'finance', 'read'), 'Author denied finance access');
      assert.ok(!acl.isAllowed('editor', 'finance', 'read'), 'Editor denied finance access');
    });

    test('moderators should have limited access to private user data', () => {
      // moderator has a deny rule with null (all privileges) on user_private_data
      // However, they inherit read access from authenticated role
      // The deny null means they cannot have ALL privileges, but can have specific ones
      assert.ok(acl.isAllowed('moderator', 'user_private_data', 'read'), 'Moderator can read private data (inherited from authenticated)');
      assert.ok(!acl.isAllowed('moderator', 'user_private_data', 'read_own'), 'Moderator denied read_own');
      assert.ok(!acl.isAllowed('moderator', 'user_private_data', null), 'Moderator denied all access (null privileges)');
    });

    test('analyst should be denied payroll access', () => {
      assert.ok(!acl.isAllowed('analyst', 'finance_payroll', 'read'), 'Analyst denied payroll access');
    });
  });

  describe('isAllowedAny with Extensive ACL', () => {
    test('should check if any role from team has permission', () => {
      // support_tier2 has escalate privilege on support resource
      assert.ok(
        acl.isAllowedAny(['support_tier1', 'support_tier2'], ['support'], ['escalate']),
        'Tier 2 from team can escalate'
      );
    });

    test('should check if any resource is accessible', () => {
      // developer can read dev_repository
      assert.ok(
        acl.isAllowedAny(['developer'], ['dev_repository', 'finance'], ['read']),
        'Developer can read at least one resource (dev_repository)'
      );
    });

    test('should check if any privilege is allowed', () => {
      // guest has read privilege on blog
      assert.ok(
        acl.isAllowedAny(['guest'], ['blog'], ['read', 'write', 'delete']),
        'Guest has at least read privilege on blog'
      );
    });
  });
});

