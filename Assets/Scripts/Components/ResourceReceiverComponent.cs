// Override resource reservations to pull a product out of
// its dispatchers resource inventory when it has become adjacent.

using System.Collections.Generic;
using Assets.Scripts.Core;
using Assets.Scripts.WorldObjects.Core;

namespace Assets.Scripts.Components.Core
{
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
            this.worldObject =
                worldObject
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Receiver component requires a parent world object"
                );
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
            this.quantity = quantity;
        }

        public void Tick()
        {
            // Only retrieve resources if we are in retrieve mode
            bool weAreInRetrieveMode =
                this.dispatchReceiver.receiverVerb
                == DispatchComponentCore.Verbs.Retrieve.ToString();
            if (!weAreInRetrieveMode)
            {
                return;
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
                return;
            }

            // Check if we already have some of the target resource
            bool weHaveTargetResource =
                this.resources.resources.GetValueOrDefault(this.targetResource, 0u) > this.quantity;
            if (weHaveTargetResource)
            {
                return;
            }

            // Check if the dispatcher has some of the target resource
            bool dispatcherHasTargetResource =
                this.dispatchReceiver.dispatcher != null
                && this.dispatchReceiver.dispatcher.worldObject != null
                && this.dispatchReceiver.dispatcher.worldObject.resources != null
                && this.dispatchReceiver.dispatcher.worldObject.resources.resources.GetValueOrDefault(
                    this.targetResource
                ) >= this.quantity;
            if (!dispatcherHasTargetResource)
            {
                return;
            }

            // Check if we have voulume capacity to receive the resource
            bool weHaveVolumeCapacity =
                this.resources.volumeCapacity > this.gameContent.Items[this.targetResource].Volume;
            if (!weHaveVolumeCapacity)
            {
                return;
            }

            // Check if we have weight capacity to receive the resource
            bool weHaveWeightCapacity =
                this.resources.weightCapacity > this.gameContent.Items[this.targetResource].Weight;
            if (!weHaveWeightCapacity)
            {
                return;
            }

            // Check if you have the energy to retrieve the resource
            bool weHaveEnergy = this.battery.Energy > 1;
            if (!weHaveEnergy)
            {
                return;
            }
            try
            {
                this.battery.Energy -= 1;
            }
            catch (BatteryComponentCore.BatteryCapacityException) { }

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
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.WorldObjects.Core;
    using Xunit;
    using Xunit.Abstractions;

    internal class TestGameContent : GameContent
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
            // Receiver
            ResourcesComponentCore receiverResources = new(new TestGameContent(), 100, 100);
            WorldObjectCore receiverWorldObject = new(null) { resources = receiverResources };
            BatteryComponentCore receiverBattery = new(100, 100);
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
                new TestGameContent(),
                "planks"
            );

            // Logic under test
            resourceReceiver.Tick();
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }

        [Fact]
        public void TestRetrieve()
        {
            // Dispatcher
            ResourcesComponentCore dispatcherResources = new(new TestGameContent(), 100, 100);
            dispatcherResources.CreateResources("planks", 1);
            WorldObjectCore dispatcherWorldObject = new(null)
            {
                resources = dispatcherResources,
                gridPosition = new(0, 0),
            };
            BatteryComponentCore dispatchBattery = new(100, 100);
            DispatchComponentCore dispatcher = new(
                dispatcherWorldObject,
                dispatchBattery,
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "planks",
                DispatchComponentCore.Keywords.Me.ToString()
            );

            // Receiver
            ResourcesComponentCore receiverResources = new(new TestGameContent(), 100, 100);
            WorldObjectCore receiverWorldObject = new(null)
            {
                resources = receiverResources,
                gridPosition = new(0, 1),
            };
            BatteryComponentCore receiverBattery = new(100, 100);
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
                new TestGameContent(),
                "planks"
            );

            // Logic under test
            resourceReceiver.Tick();

            // Assertions
            Assert.Equal(receiverResources.resources["planks"], 1u);
        }

        [Fact]
        public void TestDoesNotDuplicate()
        {
            // Dispatcher
            ResourcesComponentCore dispatcherResources = new(new TestGameContent(), 100, 100);
            dispatcherResources.CreateResources("planks", 1);
            WorldObjectCore dispatcherWorldObject = new(null)
            {
                resources = dispatcherResources,
                gridPosition = new(0, 0),
            };
            BatteryComponentCore dispatchBattery = new(100, 100);
            DispatchComponentCore dispatcher = new(
                dispatcherWorldObject,
                dispatchBattery,
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "planks",
                DispatchComponentCore.Keywords.Me.ToString()
            );

            // Receiver
            ResourcesComponentCore receiverResources = new(new TestGameContent(), 100, 100);
            WorldObjectCore receiverWorldObject = new(null)
            {
                resources = receiverResources,
                gridPosition = new(0, 1),
            };
            BatteryComponentCore receiverBattery = new(100, 100);
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
                new TestGameContent(),
                "planks"
            );

            // Logic under test
            resourceReceiver.Tick();
            resourceReceiver.Tick();
            resourceReceiver.Tick();

            // Assertions
            Assert.Equal(receiverResources.resources["planks"], 1u);
        }
    }
}
