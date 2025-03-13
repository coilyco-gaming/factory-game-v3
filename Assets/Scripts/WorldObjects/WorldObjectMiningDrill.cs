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
        public uint totalVolumeCapacity = 5000;
        public uint totalBatteryCapacity = 1000;
        public uint insertionRate = 5;
        public override float ZIndex => 2; // TODO: make this a constant

        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = Util.HumanizedString(this.WorldObjectType),
                Energy = this.core.battery.PercentEnergyStatus,
                Dispatchers = this
                    .core.dispatchers.Select(dispatcher => dispatcher.Description)
                    .ToList(),
                Receivers = this
                    .core.dispatchReceivers.Select(receiver => receiver.Description)
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
                volumeCapacity: this.totalVolumeCapacity
            );
            this.core.battery = new(capacity: this.totalBatteryCapacity);
            this.core.dispatchers = new List<DispatchComponentCore>
            {
                // TODO: adjacent stone mining drill if necessary
                new(
                    this.core,
                    this.core.battery,
                    this.core.resources,
                    new FactoryGameContent(),
                    // Retrieve...
                    DispatchComponentCore.Verbs.Retrieve.ToString(),
                    // ...< product >...
                    this.core.targetType,
                    // ...from me.
                    DispatchComponentCore.Keywords.Me.ToString()
                ),
            };
            this.core.dispatchReceivers = new List<DispatchReceiverComponentCore>
            {
                new(
                    this.core,
                    this.core.resources,
                    DispatchComponentCore.Verbs.Stockpile.ToString(),
                    this.core.targetType
                ),
            };
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.battery.Balance(this.core, gameController.core);
            foreach (DispatchComponentCore dispatcher in this.core.dispatchers)
            {
                this.core.Alerts = dispatcher.Tick(gameController.core);
            }
        }
    }
}
