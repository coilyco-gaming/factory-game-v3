using Assets.Scripts.WorldObjects.Core;

namespace Assets.Scripts.Components.Core
{
    public class DispatchReceiverComponentCore
    {
        public bool awaitingTarget = true;
        public WorldObjectCore worldObject;
        public DispatchComponentCore dispatchHQ;
        public System.Numerics.Vector2 targetPosition;

        public DispatchReceiverComponentCore(WorldObjectCore worldObject)
        {
            this.worldObject = worldObject;
        }

        public void Tick() { }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class DispatchReceiverComponentTest
    {
        [Fact]
        public void TestTrue()
        {
            WorldObjectCore worldObject = new(null);
            DispatchReceiverComponentCore receiver = new(worldObject);
            receiver.Tick();
            Assert.True(true);
        }
    }
}
