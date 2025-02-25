namespace Assets.Scripts.Components.Core
{
    public class MovementComponentCore
    {
        public void Instantiate() { }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using Assets.Scripts.Components.Core;
    using UnityEngine;

    public class MovementComponent : MonoBehaviour
    {
        public readonly MovementComponentCore core = new();

        public void Instantiate() => this.core.Instantiate();
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class MovementComponentTest
    {
        [Fact]
        public void TestTrue()
        {
            MovementComponentCore MovementComponent = new();
            MovementComponent.Instantiate();
            Assert.True(true);
        }
    }
}
