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
        public readonly ExampleComponentCore core = new();

        public void Instantiate() => this.core.Instantiate();
    }
}
#endif

// TODO: add tree shaking to remove the tests from the build
namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;
    using Xunit.Abstractions;

    public class ExampleComponentTest
    {
        private ITestOutputHelper testOutput;

        public ExampleComponentTest(ITestOutputHelper testOutputHelper)
        {
            this.testOutput = testOutputHelper;
        }

        [Fact]
        public void TestTrue()
        {
            ExampleComponentCore ExampleComponent = new();
            ExampleComponent.Instantiate();
            Assert.True(true);
        }
    }
}
