#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using System;
    using System.Collections.Generic;
    using Assets.Scripts.WorldObjects.Core;
    using TMPro;
    using UnityEngine;
    using YamlDotNet.Serialization;
    using YamlDotNet.Serialization.NamingConventions;

    public class StatusUIRightComponent : MonoBehaviour
    {
        // FIELDS //

        private TextMeshProUGUI textMeshPro;
        private ISerializer serializer = new SerializerBuilder()
            .WithNamingConvention(PascalCaseNamingConvention.Instance)
            .ConfigureDefaultValuesHandling(DefaultValuesHandling.OmitNull)
            .Build();

        // FUNCTIONS //

        private class StatusData
        {
            public float BatteryEnergy { get; set; } = 0;
            public Dictionary<string, uint> Objects { get; set; } = new();
            public Dictionary<string, uint> Resources { get; set; } = new();
        }

        public void Instantiate()
        {
            this.textMeshPro = this.transform.GetComponent<TextMeshProUGUI>();
        }

        public void Display(List<WorldObjectCore> worldObjects)
        {
            StatusData data = new();

            // Total battery energy, via Energy attribute
            worldObjects?.ForEach(worldObject =>
            {
                if (worldObject.battery != null)
                {
                    data.BatteryEnergy += worldObject.battery.Energy;
                }
            });
            data.BatteryEnergy = (float)Math.Round(data.BatteryEnergy, 0);

            worldObjects?.ForEach(worldObject =>
            {
                // Count objects
                if (!data.Objects.ContainsKey(worldObject.worldObjectType))
                {
                    data.Objects.Add(worldObject.worldObjectType, 0);
                }
                data.Objects[worldObject.worldObjectType] += 1;

                // Count resources
                if (worldObject.resources != null)
                {
                    foreach (KeyValuePair<string, uint> resource in worldObject.resources.resources)
                    {
                        if (!data.Resources.ContainsKey(resource.Key))
                        {
                            data.Resources.Add(resource.Key, 0);
                        }
                        data.Resources[resource.Key] += resource.Value;
                    }
                }
            });

            // Serialize the status list to YAML, then update the status
            string statusYaml = this.serializer.Serialize(data);
            this.textMeshPro.SetText(statusYaml);
        }
    }
}
#endif
