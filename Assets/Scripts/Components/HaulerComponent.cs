namespace Assets.Scripts.Components.Core
{
    public class HaulerComponentCore
    {
        public void Instantiate() { }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using Assets.Scripts.Components.Core;
    using UnityEngine;

    public class HaulerComponent : MonoBehaviour
    {
        public readonly HaulerComponentCore core = new();

        public void Instantiate() => this.core.Instantiate();
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class HaulerComponentTest
    {
        [Fact]
        public void TestTrue()
        {
            HaulerComponentCore HaulerComponent = new();
            HaulerComponent.Instantiate();
            Assert.True(true);
        }
    }
}
