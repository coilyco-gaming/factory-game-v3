namespace Assets.Scripts.UI
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using TMPro;
    using UnityEngine;
    using YamlDotNet.Serialization;
    using YamlDotNet.Serialization.NamingConventions;

    public class StatusUILeft : MonoBehaviour
    {
        private TextMeshProUGUI textMeshPro;
        private ISerializer serializer = new SerializerBuilder()
            .WithNamingConvention(PascalCaseNamingConvention.Instance)
            .ConfigureDefaultValuesHandling(DefaultValuesHandling.OmitNull)
            .Build();

        private class StatusUILeftData
        {
            public string Position { get; set; }
            public List<StatusDataComponentCore> Status { get; set; }
        }

        public void Instantiate()
        {
            this.textMeshPro = this.transform.GetComponent<TextMeshProUGUI>();
        }

        public void Display(
            IEnumerable<WorldObjectCore> worldObjects,
            System.Numerics.Vector2 position
        )
        {
            // Get the status data from each object
            List<StatusDataComponentCore> statusDataList = worldObjects
                ?.Where(worldObject => worldObject != null)
                ?.Where(worldObject => worldObject.backref != null)
                ?.Select(worldObject => worldObject.backref)
                ?.Select(worldObject => worldObject.StatusData)
                .ToList();

            StatusUILeftData data = new()
            {
                Position = position.ToString(),
                Status =
                    statusDataList != null && statusDataList.Count() != 0 ? statusDataList : null,
            };

            // Serialize the status list to YAML, then update the status
            string statusYaml = this.serializer.Serialize(data);
            this.textMeshPro.SetText(statusYaml);
        }
    }
}
