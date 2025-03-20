using System;
using System.Collections.Generic;
using System.Diagnostics;
using Assets.Scripts.Core;
using UnityEngine;

namespace Assets.Scripts.Components.Core
{
    [Serializable]
    public class DispatchReceiverComponentCore
    {
        public WorldObjectCore worldObject;
        public DispatchComponentCore dispatcher;
        public System.Numerics.Vector2? targetPosition = null;
        public Dictionary<
            DispatchComponentCore,
            Tuple<uint, System.Numerics.Vector2>
        > dispatchHistory = new();
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
            string receiverVerb = "",
            string receiverSubject = ""
        )
        {
            this.worldObject = worldObject;
            this.receiverVerb = receiverVerb;
            this.receiverSubject = receiverSubject;
        }

        public void QueueDispatch(
            DispatchComponentCore dispatcher,
            System.Numerics.Vector2 targetPosition,
            GameControllerCore gameController
        )
        {
            this.dispatchHistory[dispatcher] = new(
                gameController.backref.TickCount,
                targetPosition
            );
            this.dispatcher = dispatcher;
            this.targetPosition = targetPosition;
        }

        public List<Dictionary<uint, string>> Tick(
            GameControllerCore gameController,
            WorldObjectCore worldObject
        )
        {
            using Activity activity = gameController.backref.ActivitySource.StartActivity(
                this.GetType().Name
            );
            activity.SetTag("WorldObjectType", worldObject.worldObjectType);
            activity.SetTag("tick", gameController.backref.TickCount);

            // TODO: not this
            this.worldObject = worldObject;

            bool hasTargetItem =
                worldObject.resources.resources.GetValueOrDefault(this.receiverSubject, 0u) > 0;
            // If your job is to retrieve something and you have it, switch to deploy
            if (this.receiverVerb == DispatchComponentCore.Verbs.Retrieve.ToString())
            {
                if (hasTargetItem)
                {
                    this.SwapTo(DispatchComponentCore.Verbs.Deploy);
                    return new()
                    {
                        new() { { gameController.backref.TickCount, "retrieve => deploy" } },
                    };
                }
            }
            // If your job is to deploy and you have no more of the target item, switch to retrieve
            if (this.receiverVerb == DispatchComponentCore.Verbs.Deploy.ToString())
            {
                if (!hasTargetItem)
                {
                    this.SwapTo(DispatchComponentCore.Verbs.Retrieve);
                    return new()
                    {
                        new() { { gameController.backref.TickCount, "deploy => retrieve" } },
                    };
                }
            }
            // If your job is to collect and you have the target item, switch to Deliver
            if (this.receiverVerb == DispatchComponentCore.Verbs.Collect.ToString())
            {
                if (hasTargetItem)
                {
                    this.SwapTo(DispatchComponentCore.Verbs.Deliver);
                    return new()
                    {
                        new() { { gameController.backref.TickCount, "collect => deliver" } },
                    };
                }
            }
            // If your job is to Deliver and you have no more of the target item, switch to collect
            if (this.receiverVerb == DispatchComponentCore.Verbs.Deliver.ToString())
            {
                if (!hasTargetItem)
                {
                    this.SwapTo(DispatchComponentCore.Verbs.Collect);
                    return new()
                    {
                        new() { { gameController.backref.TickCount, "deliver => collect" } },
                    };
                }
            }

            return new()
            {
                new()
                {
                    {
                        gameController.backref.TickCount,
                        $"{this.Description}: receiver state valid"
                    },
                },
            };
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
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };
            WorldObjectCore worldObject = new(null);
            ResourcesComponentCore resources = new(new TestGameContent(), 100, 100);
            worldObject.resources = resources;
            DispatchReceiverComponentCore receiver = new(worldObject, "FIGHT", "DINOSAURS");
            receiver.Tick(gameController, worldObject);
            Assert.True(true);
        }

        [Fact]
        public void TestSwapsToDeploy()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };
            WorldObjectCore worldObject = new(null);
            ResourcesComponentCore resources = new(new TestGameContent(), 100, 100);
            resources.resources["planks"] = 10;
            worldObject.resources = resources;
            DispatchReceiverComponentCore receiver = new(
                worldObject,
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "planks"
            );
            receiver.Tick(gameController, worldObject);
            Assert.Equal(DispatchComponentCore.Verbs.Deploy.ToString(), receiver.receiverVerb);
        }
    }
}
