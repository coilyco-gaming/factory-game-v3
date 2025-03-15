using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectMiningDrill : WorldObject
    {
        private static uint totalVolumeCapacity = 5000;
        private static uint totalBatteryCapacity = 1000;
        private static int miningSpeed = 5;
        private static int miningEnergyCost = 5;
        public override float ZIndex => 2; // TODO: make this a constant

        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = Util.HumanizedString(this.WorldObjectType),
                Energy = this.core.battery.PercentEnergyStatus,
                Dispatchers = this
                    .core.dispatchers.Select(dispatcher => dispatcher.Description)
                    .ToList(),
                Resources = this.core.resources.ResourceInfo,
                Info = new()
                {
                    { "Outputs", Util.HumanizedString(this.core.targetType).ToLower() },
                    { "Storage Volume", this.core.resources.UsedVolumeString },
                },
                Alerts = this.core.Alerts.Count == 0 ? null : this.core.Alerts,
            };

        public override void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            base.Instantiate(spawnQueueItem, gameContent);
            this.core.passThrough = true;
            this.core.resources = new(
                gameContent,
                weightCapacity: uint.MaxValue,
                volumeCapacity: WorldObjectMiningDrill.totalVolumeCapacity
            )
            {
                resources = spawnQueueItem.resources,
            };
            this.core.battery = new(capacity: WorldObjectMiningDrill.totalBatteryCapacity);
            this.core.dispatchers = new List<DispatchComponentCore>
            {
                // TODO: adjacent stone mining drill if necessary
                new(
                    this.core,
                    this.core.battery,
                    this.core.resources,
                    gameContent,
                    // Collect...
                    DispatchComponentCore.Verbs.Collect.ToString(),
                    // ...< product >...
                    this.core.targetType,
                    // ...from me.
                    DispatchComponentCore.Keywords.Me.ToString()
                ),
            };
            this.core.mining = new MiningComponentCore(
                this.core,
                gameContent,
                this.core.targetType,
                WorldObjectMiningDrill.miningSpeed,
                WorldObjectMiningDrill.miningEnergyCost
            );
            this.core.powerLine = new PowerLineComponentCore(
                this.core,
                FactoryGameContent.Spawnables.PowerLines.ToString()
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.battery.Balance(this.core, gameController.core);
            foreach (DispatchComponentCore dispatcher in this.core.dispatchers)
            {
                this.core.Alerts = dispatcher.Tick(gameController.core);
            }
            this.core.Alerts = this.core.mining.Tick(gameController.core);
            this.core.powerLine.Tick(gameController.core);

            // If no ore world object is on our position
            bool oreAtPosition =
                gameController
                    .core.worldObjects.GetValueOrDefault(this.GridPosition)
                    ?.Any(worldObject => worldObject.Value.backref is WorldObjectOre ore) ?? false;

            // If our resources are empty
            bool resourcesEmpty = !this.core.resources.HasResources;

            // Then delete the mining drill
            if (!oreAtPosition && resourcesEmpty)
            {
                gameController.QueueForDeletion(
                    new DeletionQueueItem(this.core, this.GridPosition)
                );
            }
        }
    }
}
