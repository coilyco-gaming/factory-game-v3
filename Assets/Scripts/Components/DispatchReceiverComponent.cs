using System.Collections.Generic;
using Assets.Scripts.Core;

namespace Assets.Scripts.Components.Core
{
    public class DispatchReceiverComponentCore
    {
        public WorldObjectCore worldObject;
        public DispatchComponentCore dispatcher;
        public System.Numerics.Vector2? targetPosition = null;
        private ResourcesComponentCore resources;
        private string DescriptionToOrFrom =>
            this.receiverVerb == DispatchComponentCore.Verbs.Retrieve.ToString()
            || this.receiverVerb == DispatchComponentCore.Verbs.Collect.ToString()
                ? " from"
            : this.receiverVerb == DispatchComponentCore.Verbs.Deliver.ToString() ? ""
            : " to";
        public string Description =>
            this.dispatcher != null
                ? $"{this.receiverVerb} {this.DescriptionSubject}{this.DescriptionToOrFrom}{this.TargetDescription}".ToLower()
                : $"awaiting {this.DescriptionSubject} to {this.receiverVerb}".ToLower();
        private string DescriptionSubject => Util.HumanizedString(this.receiverSubject);
        private string TargetDescription =>
            this.receiverVerb == DispatchComponentCore.Verbs.Deliver.ToString()
                ? ""
                : $" {this.targetPosition}";
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
            bool hasTargetItem =
                this.resources.resources.GetValueOrDefault(this.receiverSubject, 0u) > 0;
            // If your job is to retrieve something and you have it, switch to deploy
            if (this.receiverVerb == DispatchComponentCore.Verbs.Retrieve.ToString())
            {
                if (hasTargetItem)
                {
                    this.SwapTo(DispatchComponentCore.Verbs.Deploy);
                }
            }
            // If your job is to deploy and you have no more of the target item, switch to retrieve
            if (this.receiverVerb == DispatchComponentCore.Verbs.Deploy.ToString())
            {
                if (!hasTargetItem)
                {
                    this.SwapTo(DispatchComponentCore.Verbs.Retrieve);
                }
            }
            // If your job is to collect and you have the target item, switch to Deliver
            if (this.receiverVerb == DispatchComponentCore.Verbs.Collect.ToString())
            {
                if (hasTargetItem)
                {
                    this.SwapTo(DispatchComponentCore.Verbs.Deliver);
                }
            }
            // If your job is to Deliver and you have no more of the target item, switch to collect
            if (this.receiverVerb == DispatchComponentCore.Verbs.Deliver.ToString())
            {
                if (!hasTargetItem)
                {
                    this.SwapTo(DispatchComponentCore.Verbs.Collect);
                }
            }
        }

        private void SwapTo(DispatchComponentCore.Verbs verb)
        {
            if (this.dispatcher != null)
            {
                this.dispatcher.receiver = null;
            }
            this.dispatcher = null;
            this.targetPosition = null;
            this.receiverVerb = verb.ToString();
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

        [Fact]
        public void TestSwapsToDeploy()
        {
            WorldObjectCore worldObject = new(null);
            ResourcesComponentCore resources = new(new TestGameContent(), 100, 100);
            resources.resources["planks"] = 10;
            DispatchReceiverComponentCore receiver = new(
                worldObject,
                resources,
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "planks"
            );
            receiver.Tick();
            Assert.Equal(DispatchComponentCore.Verbs.Deploy.ToString(), receiver.receiverVerb);
        }
    }
}
