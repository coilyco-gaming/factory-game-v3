// Override resource reservations to pull a product out of
// its dispatchers resource inventory when it has become adjacent.

using System;
using System.Collections.Generic;
using System.Diagnostics;
using Assets.Scripts.Core;
using UnityEngine;

namespace Assets.Scripts.Components.Core
{
    [Serializable]
    public class ResourceRetrieverCore
    {
        private WorldObjectCore worldObject;
        private ResourcesComponentCore resources;
        private DispatchReceiverComponentCore dispatchReceiver;
        private BatteryComponentCore battery;
        private GameContent gameContent;
        private string targetResource;
        private uint quantity;

        public ResourceRetrieverCore(
            WorldObjectCore worldObject,
            ResourcesComponentCore resources,
            BatteryComponentCore battery,
            DispatchReceiverComponentCore dispatchReceiver,
            GameContent gameContent,
            string targetResource,
            uint quantity = 1
        )
        {
            this.worldObject = worldObject;
            this.resources =
                resources
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Receiver component requires a resource component"
                );
            this.battery =
                battery
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Receiver component requires a battery component"
                );
            this.dispatchReceiver =
                dispatchReceiver
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Receiver component requires a dispatch receiver component"
                );
            this.gameContent =
                gameContent
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Receiver component requires game content"
                );
            this.targetResource =
                targetResource
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Receiver component requires a target resource"
                );
            this.quantity = quantity; // TODO: make this stack size
        }

        public List<Dictionary<uint, string>> Tick(GameControllerCore gameController)
        {
            using Activity activity = gameController.backref.ActivitySource.StartActivity(
                this.GetType().Name
            );
            activity.SetTag("WorldObjectType", this.worldObject.worldObjectType);

            // Only get resources if we are in retrieve or collect mode
            bool weAreInRetrieveMode =
                this.dispatchReceiver.receiverVerb
                == DispatchComponentCore.Verbs.Retrieve.ToString();
            bool weAreInCollectMode =
                this.dispatchReceiver.receiverVerb
                == DispatchComponentCore.Verbs.Collect.ToString();
            if (!weAreInRetrieveMode && !weAreInCollectMode)
            {
                return new()
                {
                    new()
                    {
                        { gameController.backref.TickCount, "not in retrieve or collect mode" },
                    },
                };
            }

            // Only retrieve resources if we are adjacent to the dispatcher
            bool dispatcherIsAdjacent =
                this.dispatchReceiver.targetPosition != null
                && System.Numerics.Vector2.Distance(
                    this.worldObject.gridPosition,
                    this.dispatchReceiver.targetPosition.Value
                ) < 1.5;
            if (!dispatcherIsAdjacent)
            {
                return new()
                {
                    new() { { gameController.backref.TickCount, "dispatcher is not adjacent" } },
                };
            }

            // Check if we already have some of the target resource
            bool weHaveTargetResource =
                this.resources.resources.GetValueOrDefault(this.targetResource, 0u) > this.quantity;
            if (weHaveTargetResource)
            {
                return new()
                {
                    new() { { gameController.backref.TickCount, "we have resource to retrieve" } },
                };
            }

            // Retrieve the resource from the dispatcher
            try
            {
                this.resources.RetrieveResources(
                    this.dispatchReceiver.dispatcher.worldObject.resources,
                    this.targetResource,
                    this.quantity
                );
            }
            catch (ResourcesComponentCore.ResourceException) { }

            return new();
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Xunit;
    using Xunit.Abstractions;

    internal class TestResourceReceiverGameContent : GameContent
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

    public class ResourceRetrieverCoreTest
    {
        private ITestOutputHelper testOutput;

        public ResourceRetrieverCoreTest(ITestOutputHelper testOutputHelper)
        {
            this.testOutput = testOutputHelper;
        }

        [Fact]
        public void TestTrue()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            // Receiver
            ResourcesComponentCore receiverResources = new(
                new TestResourceReceiverGameContent(),
                100,
                100
            );
            WorldObjectCore receiverWorldObject = new(null) { resources = receiverResources };
            BatteryComponentCore receiverBattery = new(receiverWorldObject, 100, 100);
            DispatchReceiverComponentCore dispatchReceiver = new(
                receiverWorldObject,
                receiverResources,
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "planks"
            );
            ResourceRetrieverCore resourceReceiver = new(
                receiverWorldObject,
                receiverResources,
                receiverBattery,
                dispatchReceiver,
                new TestResourceReceiverGameContent(),
                "planks"
            );

            // Logic under test
            resourceReceiver.Tick(gameController);
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }

        [Fact]
        public void TestRetrieve()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            // Dispatcher
            ResourcesComponentCore dispatcherResources = new(
                new TestResourceReceiverGameContent(),
                100,
                100
            );
            dispatcherResources.CreateResources("planks", 1);
            WorldObjectCore dispatcherWorldObject = new(null)
            {
                resources = dispatcherResources,
                gridPosition = new(0, 0),
            };
            BatteryComponentCore dispatchBattery = new(dispatcherWorldObject, 100, 100);
            DispatchComponentCore dispatcher = new(
                dispatcherWorldObject,
                dispatchBattery,
                new ResourcesComponentCore(new TestResourceReceiverGameContent(), 100, 100),
                new TestResourceReceiverGameContent(),
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "planks",
                DispatchComponentCore.Keywords.Me.ToString()
            );

            // Receiver
            ResourcesComponentCore receiverResources = new(
                new TestResourceReceiverGameContent(),
                100,
                100
            );
            WorldObjectCore receiverWorldObject = new(null)
            {
                resources = receiverResources,
                gridPosition = new(0, 1),
            };
            BatteryComponentCore receiverBattery = new(receiverWorldObject, 100, 100);
            DispatchReceiverComponentCore dispatchReceiver = new(
                dispatcherWorldObject,
                receiverResources,
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "planks"
            )
            {
                dispatcher = dispatcher,
                targetPosition = new(0, 1),
            };
            ResourceRetrieverCore resourceReceiver = new(
                receiverWorldObject,
                receiverResources,
                receiverBattery,
                dispatchReceiver,
                new TestResourceReceiverGameContent(),
                "planks"
            );

            // Logic under test
            resourceReceiver.Tick(gameController);

            // Assertions
            Assert.Equal(receiverResources.resources["planks"], 1u);
        }

        [Fact]
        public void TestDoesNotDuplicate()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            // Dispatcher
            ResourcesComponentCore dispatcherResources = new(
                new TestResourceReceiverGameContent(),
                100,
                100
            );
            dispatcherResources.CreateResources("planks", 1);
            WorldObjectCore dispatcherWorldObject = new(null)
            {
                resources = dispatcherResources,
                gridPosition = new(0, 0),
            };
            BatteryComponentCore dispatchBattery = new(dispatcherWorldObject, 100, 100);
            DispatchComponentCore dispatcher = new(
                dispatcherWorldObject,
                dispatchBattery,
                new ResourcesComponentCore(new TestResourceReceiverGameContent(), 100, 100),
                new TestResourceReceiverGameContent(),
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "planks",
                DispatchComponentCore.Keywords.Me.ToString()
            );

            // Receiver
            ResourcesComponentCore receiverResources = new(
                new TestResourceReceiverGameContent(),
                100,
                100
            );
            WorldObjectCore receiverWorldObject = new(null)
            {
                resources = receiverResources,
                gridPosition = new(0, 1),
            };
            BatteryComponentCore receiverBattery = new(receiverWorldObject, 100, 100);
            DispatchReceiverComponentCore dispatchReceiver = new(
                dispatcherWorldObject,
                receiverResources,
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "planks"
            )
            {
                dispatcher = dispatcher,
                targetPosition = new(0, 1),
            };
            ResourceRetrieverCore resourceReceiver = new(
                receiverWorldObject,
                receiverResources,
                receiverBattery,
                dispatchReceiver,
                new TestResourceReceiverGameContent(),
                "planks"
            );

            // Logic under test
            resourceReceiver.Tick(gameController);
            resourceReceiver.Tick(gameController);
            resourceReceiver.Tick(gameController);

            // Assertions
            Assert.Equal(receiverResources.resources["planks"], 1u);
        }
    }
}
