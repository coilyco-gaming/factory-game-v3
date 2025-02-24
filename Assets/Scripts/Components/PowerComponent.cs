namespace Assets.Scripts.Components.Core
{
    public class PowerComponentCore
    {
        public void Initialize() { }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using Assets.Scripts.Components.Core;
    using UnityEngine;

    public class PowerComponent : MonoBehaviour
    {
        private PowerComponentCore PowerComponentCore { get; } = new PowerComponentCore();

        public void Initialize() => this.PowerComponentCore.Initialize();
    }
}
#endif

#if TESTS
namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class PowerComponentTest
    {
        [Fact]
        public void Test1()
        {
            PowerComponentCore powerComponentCore = new();
            powerComponentCore.Initialize();
            Assert.False(true);
        }

        [Fact]
        public void Test2()
        {
            PowerComponentCore powerComponentCore = new();
            powerComponentCore.Initialize();
            Assert.False(false);
        }
    }
}
#endif
