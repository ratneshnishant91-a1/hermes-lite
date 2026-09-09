#!/usr/bin/env python3
"""
Hermes-Lite v1.5 - Test Suite

Run: python test_hermes.py
"""

import sys
import os


def test_imports():
    """Test that all modules can be imported."""
    print("Testing imports...")
    
    modules = [
        "store",
        "workspace",
        "memory",
        "skills",
        "approval",
        "tools",
        "browser",
        "search",
        "compressor",
        "context",
        "context_selector",
        "streaming",
        "security",
        "health",
        "learner",
        "network",
        "config",
    ]
    
    failed = []
    for module in modules:
        try:
            __import__(module)
            print(f"  ✓ {module}")
        except Exception as e:
            print(f"  ✗ {module}: {e}")
            failed.append(module)
    
    if failed:
        print(f"\n❌ Failed imports: {', '.join(failed)}")
        return False
    
    print("\n✅ All imports successful\n")
    return True


def test_store():
    """Test database operations."""
    print("Testing store...")
    
    try:
        from store import Store
        store = Store("test_hermes.db")
        
        # Test session creation
        session_id = store.create_session()
        assert session_id > 0, "Session ID should be positive"
        
        # Test memory
        result = store.remember("Test memory", source="test")
        assert "Saved memory" in result, "Memory should be saved"
        
        # Test search
        memories = store.search_memories("Test", limit=5)
        assert len(memories) > 0, "Should find memories"
        
        print("  ✓ Session creation")
        print("  ✓ Memory save")
        print("  ✓ Memory search")
        
        # Cleanup
        os.remove("test_hermes.db")
        
        print("\n✅ Store tests passed\n")
        return True
        
    except Exception as e:
        print(f"\n❌ Store test failed: {e}\n")
        return False


def test_workspace():
    """Test workspace sandbox."""
    print("Testing workspace...")
    
    try:
        from workspace import Workspace
        workspace = Workspace("test_workspace")
        
        # Test path resolution
        path = workspace.resolve("test.txt")
        assert "test_workspace" in str(path), "Path should be in workspace"
        
        # Test path traversal prevention
        try:
            workspace.resolve("../etc/passwd")
            print("  ✗ Path traversal not blocked")
            return False
        except PermissionError:
            print("  ✓ Path traversal blocked")
        
        print("  ✓ Path resolution")
        
        # Cleanup
        import shutil
        shutil.rmtree("test_workspace", ignore_errors=True)
        
        print("\n✅ Workspace tests passed\n")
        return True
        
    except Exception as e:
        print(f"\n❌ Workspace test failed: {e}\n")
        return False


def test_security():
    """Test security modules."""
    print("Testing security...")
    
    try:
        from security import InputValidator, RateLimiter, AuditLogger
        
        # Test input validation
        valid, error = InputValidator.validate_user_message("Hello world")
        assert valid, "Valid message should pass"
        print("  ✓ Input validation")
        
        # Test path traversal detection
        valid, error = InputValidator.validate_user_message("../etc/passwd")
        assert not valid, "Path traversal should be blocked"
        print("  ✓ Path traversal detection")
        
        # Test rate limiter
        limiter = RateLimiter(max_requests=5, window_seconds=60)
        for i in range(5):
            allowed, _ = limiter.is_allowed("test-user")
            assert allowed, f"Request {i+1} should be allowed"
        
        allowed, wait = limiter.is_allowed("test-user")
        assert not allowed, "6th request should be blocked"
        print("  ✓ Rate limiting")
        
        # Test audit logger
        audit = AuditLogger("test_audit_logs")
        audit.log_event("TEST", {"message": "Test event"}, "test-user")
        events = audit.get_recent_events()
        assert len(events) > 0, "Should have logged events"
        print("  ✓ Audit logging")
        
        # Cleanup
        import shutil
        shutil.rmtree("test_audit_logs", ignore_errors=True)
        
        print("\n✅ Security tests passed\n")
        return True
        
    except Exception as e:
        print(f"\n❌ Security test failed: {e}\n")
        return False


def test_network():
    """Test network security."""
    print("Testing network...")
    
    try:
        from network import NetworkConfig, is_private_ip, is_domain_allowed
        
        # Test private IP detection
        assert is_private_ip("192.168.1.1"), "Should detect private IP"
        assert is_private_ip("10.0.0.1"), "Should detect private IP"
        assert is_private_ip("127.0.0.1"), "Should detect localhost"
        assert not is_private_ip("8.8.8.8"), "Should allow public IP"
        print("  ✓ SSRF protection")
        
        # Test domain allowlist
        config = NetworkConfig(allowed_domains=["api.github.com", "github.com"])
        assert is_domain_allowed("api.github.com", config.allowed_domains)
        assert is_domain_allowed("raw.githubusercontent.com", config.allowed_domains)
        assert not is_domain_allowed("evil.com", config.allowed_domains)
        print("  ✓ Domain allowlist")
        
        print("\n✅ Network tests passed\n")
        return True
        
    except Exception as e:
        print(f"\n❌ Network test failed: {e}\n")
        return False


def test_learner():
    """Test self-learning."""
    print("Testing learner...")
    
    try:
        from learner import SelfLearner
        from store import Store
        from memory import Memory
        from skills import Skills
        from workspace import Workspace
        
        store = Store("test_learner.db")
        memory = Memory(store)
        skills = Skills("test_skills")
        workspace = Workspace("test_workspace")
        
        learner = SelfLearner(store, skills, memory, workspace)
        
        # Test learning from task
        learner.learn_from_task(
            task_description="Create a Python file",
            conversation_messages=[
                {"role": "user", "content": "Create hello.py"},
                {"role": "assistant", "content": "Creating file..."},
            ],
            result="Created hello.py",
            success=True,
        )
        
        stats = learner.get_learning_stats()
        assert stats["total_learning_events"] > 0, "Should have learning events"
        print("  ✓ Learning from task")
        
        # Cleanup
        import shutil
        shutil.rmtree("test_skills", ignore_errors=True)
        shutil.rmtree("test_workspace", ignore_errors=True)
        os.remove("test_learner.db")
        if os.path.exists("test_workspace/learning_log.json"):
            os.remove("test_workspace/learning_log.json")
        
        print("\n✅ Learner tests passed\n")
        return True
        
    except Exception as e:
        print(f"\n❌ Learner test failed: {e}\n")
        return False


def main():
    """Run all tests."""
    print("=" * 60)
    print("Hermes-Lite v1.5 - Test Suite")
    print("=" * 60)
    print()
    
    results = []
    
    results.append(("Imports", test_imports()))
    results.append(("Store", test_store()))
    results.append(("Workspace", test_workspace()))
    results.append(("Security", test_security()))
    results.append(("Network", test_network()))
    results.append(("Learner", test_learner()))
    
    print("=" * 60)
    print("Test Summary")
    print("=" * 60)
    
    for name, passed in results:
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"{status} - {name}")
    
    total = len(results)
    passed = sum(1 for _, p in results if p)
    
    print(f"\nTotal: {passed}/{total} tests passed")
    
    if passed == total:
        print("\n🎉 All tests passed!")
        return 0
    else:
        print(f"\n⚠️  {total - passed} test(s) failed")
        return 1


if __name__ == "__main__":
    sys.exit(main())
