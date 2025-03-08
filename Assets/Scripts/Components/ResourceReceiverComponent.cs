namespace Assets.Scripts.Components.Core
{
    public class ResourceReceiverCore
    {
        public ResourceReceiverCore() { }

        public void Tick() { }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;
    using Xunit.Abstractions;

    public class ResourceReceiverCoreTest
    {
        private ITestOutputHelper testOutput;

        public ResourceReceiverCoreTest(ITestOutputHelper testOutputHelper)
        {
            this.testOutput = testOutputHelper;
        }

        [Fact]
        public void TestTrue()
        {
            ResourceReceiverCore example = new();
            example.Tick();
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }
    }
}
