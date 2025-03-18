using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectPowerLines : WorldObject
    {
        private uint totalBatteryCapacity = 1000;

        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = Util.HumanizedString(this.WorldObjectType),
                Energy = this.core.battery.PercentEnergyStatus,
                Alerts = this.core.alerts.Count == 0 ? null : this.core.alerts,
            };

        public override void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            base.Instantiate(spawnQueueItem, gameContent);
            this.core.passThrough = true;
            this.core.battery = new(this.core, capacity: this.totalBatteryCapacity);
            this.core.powerLine = new PowerLineComponentCore(
                this.core,
                FactoryGameContent.Spawnables.PowerLines.ToString()
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.CreateAlert(gameController.core, this.core.battery.Tick(gameController.core));
            this.core.CreateAlert(
                gameController.core,
                this.core.powerLine.Tick(gameController.core)
            );
        }
    }
}
