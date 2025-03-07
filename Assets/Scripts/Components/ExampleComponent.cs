namespace Assets.Scripts.Components.Core
{
    public class ExampleComponentCore
    {
        public ExampleComponentCore() { }

        public void Tick() { }
    }
}

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
            ExampleComponentCore example = new();
            example.Tick();
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }
    }
}
