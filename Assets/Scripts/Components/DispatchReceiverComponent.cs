using Assets.Scripts.Core;
using Assets.Scripts.WorldObjects.Core;

namespace Assets.Scripts.Components.Core
{
    public class DispatchReceiverComponentCore
    {
        public bool awaitingTarget = true;
        public WorldObjectCore worldObject;
        public DispatchComponentCore dispatcher;
        public System.Numerics.Vector2 targetPosition;
        public string receiverVerb;
        public string receiverNoun;

        public DispatchReceiverComponentCore(
            WorldObjectCore worldObject,
            string receiverVerb,
            string receiverNoun
        )
        {
            this.worldObject =
                worldObject
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Reciever component requires a parent world object"
                );
            this.receiverVerb = receiverVerb;
            this.receiverNoun = receiverNoun;
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
            DispatchReceiverComponentCore receiver = new(worldObject, "FIGHT", "DINOSAURS");
            receiver.Tick();
            Assert.True(true);
        }
    }
}
