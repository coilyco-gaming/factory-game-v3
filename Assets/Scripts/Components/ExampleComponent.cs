namespace Assets.Scripts.Components.Core
{
    public class ExampleComponentCore
    {
        public void Instantiate() { }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using Assets.Scripts.Components.Core;
    using UnityEngine;

    public class ExampleComponent : MonoBehaviour
    {
        private ExampleComponentCore ExampleComponentCore = new();

        public void Instantiate() => this.ExampleComponentCore.Instantiate();
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class ExampleComponentTest
    {
        [Fact]
        public void TestTrue()
        {
            ExampleComponentCore ExampleComponent = new();
            ExampleComponent.Instantiate();
            Assert.True(true);
        }
    }
}
