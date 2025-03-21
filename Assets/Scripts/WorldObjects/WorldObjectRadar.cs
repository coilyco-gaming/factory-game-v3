using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectRadar : WorldObject
    {
        private uint totalBatteryCapacity = 1000;

        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = Util.HumanizedString(this.WorldObjectType),
                Energy = this.core.battery.PercentEnergyStatus,
                Dispatchers = this
                    .core.dispatchers.Select(dispatcher => dispatcher.Description)
                    .ToList(),
                Alerts = this.core.alerts.Count == 0 ? null : this.core.alerts,
            };

        public override void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            base.Instantiate(spawnQueueItem, gameContent);
            this.core.battery = new(capacity: this.totalBatteryCapacity);
            this.core.dispatchers = new List<DispatchComponentCore>
            {
                new(
                    gameContent,
                    // Deploy...
                    DispatchComponentCore.Verbs.Deploy.ToString(),
                    // ...mining drill...
                    this.core.targetSubType,
                    // ...to < iron ore | copper ore | coal | etc >.
                    this.core.targetType
                ),
            };
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.CreateAlert(
                gameController.core,
                this.core.battery.Tick(gameController.core, this.core)
            );
            foreach (DispatchComponentCore dispatcher in this.core.dispatchers)
            {
                this.core.CreateAlert(
                    gameController.core,
                    dispatcher.Tick(gameController.core, this.core)
                );
            }
        }
    }
}
