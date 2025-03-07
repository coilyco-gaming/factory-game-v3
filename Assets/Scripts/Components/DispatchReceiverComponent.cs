using System.Collections.Generic;
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
        private ResourcesComponentCore resources;
        public string receiverVerb;
        public string receiverSubject;

        public DispatchReceiverComponentCore(
            WorldObjectCore worldObject,
            ResourcesComponentCore resources,
            string receiverVerb,
            string receiverSubject
        )
        {
            this.worldObject =
                worldObject
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Reciever component requires a parent world object"
                );
            this.resources =
                resources
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Reciever component requires a resource component"
                );
            this.receiverVerb = receiverVerb;
            this.receiverSubject = receiverSubject;
        }

        public void Tick()
        {
            if (this.receiverVerb == DispatchComponentCore.Verbs.Retrieve.ToString())
            {
                bool hasReceiverSubject =
                    this.resources.resources.GetValueOrDefault(this.receiverSubject, 0u) > 0;
                if (hasReceiverSubject)
                {
                    this.receiverVerb = DispatchComponentCore.Verbs.Deploy.ToString();
                }
            }
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class DispatchReceiverComponentTest
    {
        private class TestGameContent : GameContent
        {
            public override Dictionary<string, Item> Items { get; } =
                new()
                {
                    { "wood", new Item("wood", stackSize: 100) },
                    {
                        "planks",
                        new Item(
                            "planks",
                            stackSize: 10,
                            craftTime: 3,
                            ingredients: new Dictionary<string, uint> { { "wood", 5 } }
                        )
                    },
                };
        }

        [Fact]
        public void TestTrue()
        {
            WorldObjectCore worldObject = new(null);
            ResourcesComponentCore resources = new(new TestGameContent(), 100, 100);
            DispatchReceiverComponentCore receiver = new(
                worldObject,
                resources,
                "FIGHT",
                "DINOSAURS"
            );
            receiver.Tick();
            Assert.True(true);
        }
    }
}
